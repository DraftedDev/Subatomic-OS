use crate::{
    api,
    device::pci::{
        InterruptStatus, PciDevice, PciDriver,
        classes::{Class, SerialBusSubClass},
        error::PciError,
    },
    sync::mutex::Mutex,
};

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

        unsafe {
            let usbcmd_ptr = operational_offset as *mut u32;
            let usbsts_ptr = (operational_offset + 0x04) as *const u32;

            let mut usbcmd = core::ptr::read_volatile(usbcmd_ptr);
            usbcmd &= !(1 << 0); // Clear R/S bit
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
        }

        self.state.run(|s| {
            *s = Some(XhciState {
                mmio_virt,
                mmio_len: bar_size,
                operational_offset,
            });
        });

        if let Some(vector) = msi_vector {
            log::info!("xHCI bound with MSI vector: {vector}");
        }

        Ok(())
    }

    fn handle_interrupt(&self, _: &PciDevice) -> InterruptStatus {
        InterruptStatus::Handled
    }

    fn destroy(&self, _: &PciDevice) -> Result<(), PciError> {
        self.state.run(|s| *s = None);
        Ok(())
    }
}

/// The internal XHCI controller state.
pub struct XhciState {
    /// The internal state MMIO virtual address.
    pub mmio_virt: usize,
    /// The internal state MMIO size.
    pub mmio_len: usize,
    /// The internal state operational offset.
    pub operational_offset: usize,
}
