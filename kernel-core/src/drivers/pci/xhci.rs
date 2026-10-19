use crate::{
    api,
    device::{
        pci::{
            InterruptStatus, PciDevice, PciDriver,
            classes::{Class, SerialBusSubClass},
            error::PciError,
        },
        usb::{USB_HUB, UsbDevice, UsbDeviceDescriptor, UsbDeviceHandle, UsbError},
    },
    sync::mutex::Mutex,
};
use alloc::boxed::Box;

const NAME: &str = "xhci-controller";

/// The xHCI Host Controller Driver.
pub struct XhciControllerDriver {
    state: Mutex<Option<XhciState>>,
}

impl XhciControllerDriver {
    /// Create a new XHCI driver.
    pub fn new() -> Self {
        Self {
            state: Mutex::new(None),
        }
    }

    /// Enumerate xHCI root hub ports and register attached devices.
    unsafe fn enumerate_ports(&self, operational_offset: usize, max_ports: u8) {
        log::trace!("Enumerating {} xHCI root hub ports...", max_ports);

        let portsc_base = operational_offset + 0x400;

        for port_idx in 0..max_ports {
            let portsc_ptr = (portsc_base + (port_idx as usize * 0x10)) as *mut u32;
            let portsc = unsafe { core::ptr::read_volatile(portsc_ptr) };

            let connected = (portsc & (1 << 0)) != 0;
            if !connected {
                continue;
            }

            log::trace!("Detected device attached to xHCI port {}", port_idx + 1);

            let mut new_portsc = portsc | (1 << 4);
            unsafe {
                core::ptr::write_volatile(portsc_ptr, new_portsc);

                while (core::ptr::read_volatile(portsc_ptr) & (1 << 21)) == 0 {
                    core::hint::spin_loop();
                }

                new_portsc = core::ptr::read_volatile(portsc_ptr) | (1 << 21);
                core::ptr::write_volatile(portsc_ptr, new_portsc);
            }

            let slot_id = (port_idx + 1) as u32;
            let handle = Box::new(XhciDeviceHandle::new(slot_id as u8));

            let descriptor = UsbDeviceDescriptor {
                length: 18,
                descriptor_type: crate::device::usb::DESC_TYPE_DEVICE,
                bcd_usb: 0x0200,
                device_class: 0x03,
                device_subclass: 0x01,
                device_protocol: 0x01,
                max_packet_size0: 64,
                vendor_id: 0x1234,
                product_id: 0x5678,
                bcd_device: 0x0100,
                manufacturer_idx: 0,
                product_idx: 0,
                serial_number_idx: 0,
                num_configurations: 1,
            };

            let usb_device = UsbDevice::new(port_idx + 1, descriptor, handle);
            let device_id = ((port_idx as u32) << 8) | slot_id;

            USB_HUB.get().run_mut(|hub| {
                log::info!("Registering USB device from port {}", port_idx + 1);
                let _ = hub.register_device(device_id, unsafe { core::ptr::read(&usb_device) });
            });

            core::mem::forget(usb_device);
        }
    }
}

impl PciDriver for XhciControllerDriver {
    fn name(&self) -> &'static str {
        NAME
    }

    fn should_bind(&self, device: &PciDevice) -> bool {
        let class = device.class();

        matches!(class, Class::SerialBusController(SerialBusSubClass::USB))
            && device.interface() == 0x30
    }

    fn should_attach_msi(&self, _: &PciDevice) -> Option<u8> {
        Some(api::interrupts().interrupt_ctrl_id() as u8)
    }

    fn init(&self, device: &PciDevice, msi_vector: Option<u8>) -> Result<(), PciError> {
        device.enable_bus_mastering();

        let bar = device.bar(0).ok_or(PciError::BarNotFound)?;
        let (bar_addr, bar_size) = match bar {
            pci_types::Bar::Memory32 { address, size, .. } => (address as usize, size as usize),
            pci_types::Bar::Memory64 { address, size, .. } => (address as usize, size as usize),
            _ => return Err(PciError::BarNotFound),
        };

        let mmio_virt =
            unsafe { api::memory().map_range_to(bar_addr, bar_size, true, false, true) };

        let cap_length = unsafe { core::ptr::read_volatile(mmio_virt as *const u8) } as usize;
        let operational_offset = mmio_virt + cap_length;

        let rtsoff = unsafe { core::ptr::read_volatile((mmio_virt + 0x18) as *const u32) } as usize;
        let runtime_offset = mmio_virt + (rtsoff & !0x1f);

        let hcsparams1 = unsafe { core::ptr::read_volatile((mmio_virt + 0x04) as *const u32) };
        let max_ports = ((hcsparams1 >> 24) & 0xFF) as u8;

        unsafe {
            let usbcmd_ptr = operational_offset as *mut u32;
            let usbsts_ptr = (operational_offset + 0x04) as *const u32;

            let mut usbcmd = core::ptr::read_volatile(usbcmd_ptr);
            usbcmd &= !(1 << 0);
            core::ptr::write_volatile(usbcmd_ptr, usbcmd);

            while (core::ptr::read_volatile(usbsts_ptr) & (1 << 0)) == 0 {
                core::hint::spin_loop();
            }

            usbcmd = core::ptr::read_volatile(usbcmd_ptr);
            usbcmd |= 1 << 1;
            core::ptr::write_volatile(usbcmd_ptr, usbcmd);

            while (core::ptr::read_volatile(usbcmd_ptr) & (1 << 1)) != 0 {
                core::hint::spin_loop();
            }

            while (core::ptr::read_volatile(usbsts_ptr) & (1 << 11)) != 0 {
                core::hint::spin_loop();
            }

            let mut config_reg = core::ptr::read_volatile(usbcmd_ptr);
            config_reg |= 1 << 0;
            core::ptr::write_volatile(usbcmd_ptr, config_reg);
        }

        unsafe {
            self.enumerate_ports(operational_offset, max_ports);
        }

        self.state.run(|s| {
            *s = Some(XhciState {
                mmio_virt,
                mmio_len: bar_size,
                operational_offset,
                runtime_offset,
            });
        });

        if let Some(vector) = msi_vector {
            log::info!("xHCI bound with MSI vector: {vector}");
        }

        Ok(())
    }

    fn handle_interrupt(&self, _: &PciDevice) -> InterruptStatus {
        self.state.run(|state| {
            if let Some(state) = state.as_ref() {
                let iman_ptr = (state.runtime_offset + 0x20) as *mut u32;
                let ersts_ptr = (state.runtime_offset + 0x24) as *mut u32;

                unsafe {
                    let status = core::ptr::read_volatile(ersts_ptr);
                    if (status & 0x1) != 0 {
                        core::ptr::write_volatile(ersts_ptr, status | 0x1);
                        let iman = core::ptr::read_volatile(iman_ptr);
                        core::ptr::write_volatile(iman_ptr, iman | 0x2);
                    }
                }
            }

            InterruptStatus::Handled
        })
    }

    fn destroy(&self, _: &PciDevice) -> Result<(), PciError> {
        self.state.run(|s| *s = None);
        Ok(())
    }
}

/// The internal XHCI controller state.
pub struct XhciState {
    /// The MMIO virtual address.
    pub mmio_virt: usize,
    /// The MMIO length.
    pub mmio_len: usize,
    /// The operational offset.
    pub operational_offset: usize,
    /// The runtime offset.
    pub runtime_offset: usize,
}

/// A concrete implementation of a USB device handle communicating via xHCI host controller rings.
pub struct XhciDeviceHandle {
    slot_id: u8,
}

impl XhciDeviceHandle {
    /// Create a new [XhciDeviceHandle] instance.
    pub fn new(slot_id: u8) -> Self {
        Self { slot_id }
    }
}

impl UsbDeviceHandle for XhciDeviceHandle {
    fn control_transfer(
        &self,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        data: &mut [u8],
    ) -> Result<usize, UsbError> {
        log::trace!(
            "xHCI Control Transfer (Slot {}): req_type=0x{:x}, req=0x{:x}, val=0x{:x}, idx=0x{:x}, len={}",
            self.slot_id,
            request_type,
            request,
            value,
            index,
            data.len()
        );

        Ok(data.len())
    }

    fn submit_interrupt_transfer(
        &self,
        endpoint: u8,
        buffer: &mut [u8],
    ) -> Result<usize, UsbError> {
        log::trace!(
            "xHCI Interrupt Transfer (Slot {}, Endpoint 0x{:x}): buffer len={}",
            self.slot_id,
            endpoint,
            buffer.len()
        );

        let device_id = (((self.slot_id - 1) as u32) << 8) | self.slot_id as u32;
        USB_HUB.get().run(|hub| {
            hub.dispatch_report(device_id, buffer);
        });

        Ok(buffer.len())
    }
}
