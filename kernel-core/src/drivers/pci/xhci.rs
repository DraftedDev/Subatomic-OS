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
use alloc::vec::Vec;

const NAME: &str = "xhci-controller";

const TRB_TYPE_TRANSFER_EVENT: u32 = 32;
const TRB_TYPE_COMMAND_COMPLETION: u32 = 33;

#[inline]
fn write_reg64(addr: usize, val: u64) {
    let low = val as u32;
    let high = (val >> 32) as u32;
    unsafe {
        core::ptr::write_volatile(addr as *mut u32, low);
        core::ptr::write_volatile((addr + 4) as *mut u32, high);
    }
}

/// A Transfer Request Block (TRB) in xHCI.
#[repr(C, align(16))]
#[derive(Clone, Copy, Default, Debug)]
pub struct Trb {
    /// TRB parameter field (e.g. buffer address or setup packet).
    pub parameter: u64,
    /// TRB status field (e.g. transfer length, completion code).
    pub status: u32,
    /// TRB control field (e.g. cycle bit, TRB type, flags).
    pub control: u32,
}

/// An Event Ring Segment Table (ERST) entry in xHCI.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct ErstEntry {
    /// 64-byte aligned base address of the Event Ring Segment.
    pub ring_segment_base_address: u64,
    /// Size of the Event Ring Segment in number of TRBs.
    pub ring_segment_size: u32,
    /// Reserved field.
    pub reserved: u32,
}

/// A circular ring of TRBs (used for Command Ring and Transfer Rings).
pub struct Ring {
    /// Physical address of the ring.
    pub phys: usize,
    /// Virtual address of the ring.
    pub virt: usize,
    /// Number of TRBs in the ring.
    pub len: usize,
    /// Current enqueue index.
    pub enqueue: usize,
    /// Current producer cycle state bit.
    pub pcs: u32,
}

impl Ring {
    /// Creates a new [Ring] initialized with a Link TRB at the end.
    pub fn new(phys: usize, virt: usize, len: usize) -> Self {
        let link_ptr = (virt + (len - 1) * 16) as *mut Trb;
        unsafe {
            core::ptr::write_volatile(
                link_ptr,
                Trb {
                    parameter: phys as u64,
                    status: 0,
                    control: (6 << 10) | (1 << 1), // Type 6 (Link), Toggle Cycle (TC) = 1
                },
            );
        }

        Self {
            phys,
            virt,
            len,
            enqueue: 0,
            pcs: 1,
        }
    }

    /// Enqueues a TRB onto the ring, advancing enqueue pointer and toggling cycle on wrap.
    pub fn enqueue(&mut self, mut trb: Trb) -> usize {
        let trb_phys = self.phys + self.enqueue * 16;
        let trb_ptr = (self.virt + self.enqueue * 16) as *mut Trb;
        trb.control = (trb.control & !1) | self.pcs;
        unsafe {
            core::ptr::write_volatile(trb_ptr, trb);
        }

        self.enqueue += 1;
        if self.enqueue == self.len - 1 {
            let link_ptr = (self.virt + (self.len - 1) * 16) as *mut Trb;
            let link_trb = Trb {
                parameter: self.phys as u64,
                status: 0,
                control: (6 << 10) | (1 << 1) | self.pcs,
            };
            unsafe {
                core::ptr::write_volatile(link_ptr, link_trb);
            }
            self.enqueue = 0;
            self.pcs ^= 1;
        }

        trb_phys
    }
}

/// Event ring consumer in xHCI.
pub struct EventRing {
    /// Physical address of the event ring.
    pub phys: usize,
    /// Virtual address of the event ring.
    pub virt: usize,
    /// Number of TRBs in the event ring.
    pub len: usize,
    /// Current dequeue index.
    pub dequeue: usize,
    /// Event ring consumer cycle state bit.
    pub ecs: u32,
    /// MMIO virtual address of the ERDP register.
    pub erdp_reg: usize,
}

impl EventRing {
    /// Poll the event ring for a completed TRB.
    pub fn poll(&mut self) -> Option<Trb> {
        let trb_ptr = (self.virt + self.dequeue * 16) as *const Trb;
        let trb = unsafe { core::ptr::read_volatile(trb_ptr) };
        if (trb.control & 1) != self.ecs {
            return None;
        }

        self.dequeue += 1;
        if self.dequeue == self.len {
            self.dequeue = 0;
            self.ecs ^= 1;
        }

        let erdp = (self.phys + self.dequeue * 16) as u64 | (1 << 3); // EHB = 1
        write_reg64(self.erdp_reg, erdp);

        Some(trb)
    }
}

/// Ring doorbell for a given slot and target endpoint.
fn ring_doorbell(doorbell_base: usize, target_slot: u8, target_endpoint: u8) {
    let db_ptr = (doorbell_base + target_slot as usize * 4) as *mut u32;
    unsafe {
        core::ptr::write_volatile(db_ptr, target_endpoint as u32);
    }
}

/// Helper to write a 32-bit dword in an xHCI context.
fn write_ctx_u32(ctx_virt: usize, ctx_size: usize, ctx_idx: usize, dword: usize, val: u32) {
    let offset = ctx_idx * ctx_size + dword * 4;
    unsafe {
        core::ptr::write_volatile((ctx_virt + offset) as *mut u32, val);
    }
}

/// Sends a command on the command ring and waits for completion.
fn send_command(
    cmd_ring: &mut Ring,
    doorbell_base: usize,
    event_ring: &mut EventRing,
    trb: Trb,
) -> Result<Trb, &'static str> {
    let trb_phys = cmd_ring.enqueue(trb);
    ring_doorbell(doorbell_base, 0, 0);

    let mut timeout = 2_000_000;
    while timeout > 0 {
        if let Some(ev) = event_ring.poll() {
            let trb_type = (ev.control >> 10) & 0x3F;
            if trb_type == TRB_TYPE_COMMAND_COMPLETION {
                // Command Completion Event
                if ev.parameter as usize == trb_phys {
                    let cc = (ev.status >> 24) & 0xFF;
                    if cc == 1 {
                        return Ok(ev);
                    } else {
                        log::error!("xHCI command failed with completion code {cc}");
                        return Err("Command failed");
                    }
                }
            }
        }
        core::hint::spin_loop();
        timeout -= 1;
    }
    log::error!("send_command timed out");
    Err("Command timed out")
}

/// Executes a control transfer without data stage on EP0.
fn ep0_control_transfer(
    ep0_ring: &mut Ring,
    doorbell_base: usize,
    event_ring: &mut EventRing,
    slot_id: u8,
    req_type: u8,
    req: u8,
    value: u16,
    index: u16,
) -> Result<(), &'static str> {
    // Setup Stage TRB
    let setup_param =
        (req_type as u64) | ((req as u64) << 8) | ((value as u64) << 16) | ((index as u64) << 32);
    let setup_control = (2 << 10) | (1 << 6);
    ep0_ring.enqueue(Trb {
        parameter: setup_param,
        status: 8,
        control: setup_control,
    });

    // Status Stage TRB
    let status_control = (4 << 10) | (1 << 5) | (1 << 16); // Status Stage, IOC=1, Direction=IN (1)
    let _ = ep0_ring.enqueue(Trb {
        parameter: 0,
        status: 0,
        control: status_control,
    });

    ring_doorbell(doorbell_base, slot_id, 1);

    let mut timeout = 2_000_000;
    while timeout > 0 {
        if let Some(ev) = event_ring.poll() {
            let trb_type = (ev.control >> 10) & 0x3F;
            if trb_type == TRB_TYPE_TRANSFER_EVENT {
                // Transfer Event
                let slot = ((ev.control >> 24) & 0xFF) as u8;
                let ep_id = ((ev.control >> 16) & 0x1F) as u8;
                if slot == slot_id && ep_id == 1 {
                    let cc = (ev.status >> 24) & 0xFF;
                    if cc == 1 || cc == 13 {
                        return Ok(());
                    } else {
                        log::error!("EP0 control transfer failed with CC {cc}");
                        return Err("Control transfer failed");
                    }
                }
            }
        }
        core::hint::spin_loop();
        timeout -= 1;
    }
    Err("Control transfer timed out")
}

/// Tracks an active keyboard device attached to an xHCI slot.
pub struct KeyboardSlot {
    /// Slot ID allocated by xHCI.
    pub slot_id: u8,
    /// Device ID used in USB hub.
    pub device_id: u32,
    /// Transfer ring for EP0 (Control endpoint).
    pub ep0_ring: Ring,
    /// Transfer ring for EP1 (Interrupt IN endpoint).
    pub ep1_ring: Ring,
    /// Physical address of the report DMA buffer.
    pub report_phys: usize,
    /// Virtual address of the report DMA buffer.
    pub report_virt: usize,
}

/// The internal xHCI controller state.
pub struct XhciState {
    /// MMIO virtual base address.
    pub mmio_virt: usize,
    /// MMIO region length.
    pub mmio_len: usize,
    /// Offset to operational registers.
    pub operational_offset: usize,
    /// Offset to runtime registers.
    pub runtime_offset: usize,
    /// Offset to doorbell array registers.
    pub doorbell_offset: usize,
    /// The host controller command ring.
    pub cmd_ring: Ring,
    /// The host controller event ring.
    pub event_ring: EventRing,
    /// Active keyboard devices attached.
    pub keyboards: Vec<KeyboardSlot>,
}

impl XhciState {
    /// Polls pending events and dispatches keyboard reports.
    pub fn poll_events(&mut self) {
        while let Some(ev) = self.event_ring.poll() {
            let trb_type = (ev.control >> 10) & 0x3F;
            if trb_type == TRB_TYPE_TRANSFER_EVENT {
                // Transfer Event
                let slot = ((ev.control >> 24) & 0xFF) as u8;
                let ep_id = ((ev.control >> 16) & 0x1F) as u8;
                if ep_id == 3 {
                    // EP1 IN
                    if let Some(kbd) = self.keyboards.iter_mut().find(|k| k.slot_id == slot) {
                        let report_slice =
                            unsafe { core::slice::from_raw_parts(kbd.report_virt as *const u8, 8) };

                        USB_HUB.get().run(|hub| {
                            hub.dispatch_report(kbd.device_id, report_slice);
                        });

                        // Re-queue Normal TRB on ep1_ring
                        let normal_control = (1 << 10) | (1 << 5) | (1 << 2);
                        kbd.ep1_ring.enqueue(Trb {
                            parameter: kbd.report_phys as u64,
                            status: 8,
                            control: normal_control,
                        });

                        // Ring Doorbell for EP1 IN
                        ring_doorbell(self.doorbell_offset, kbd.slot_id, 3);
                    }
                }
            }
        }
    }
}

/// Global xHCI controller state.
pub static GLOBAL_XHCI: Mutex<Option<XhciState>> = Mutex::new(None);

/// Global poller function executed by the kernel scheduler.
pub fn poll_xhci_events() -> bool {
    GLOBAL_XHCI.run_irq(|s| {
        if let Some(state) = s.as_mut() {
            state.poll_events();
        }
    });
    true
}

/// The xHCI Host Controller Driver.
pub struct XhciControllerDriver;

impl XhciControllerDriver {
    /// Create a new XHCI driver.
    pub fn new() -> Self {
        Self
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

        let dboff = unsafe { core::ptr::read_volatile((mmio_virt + 0x14) as *const u32) } as usize;
        let doorbell_offset = mmio_virt + (dboff & !0x3);

        let hcsparams1 = unsafe { core::ptr::read_volatile((mmio_virt + 0x04) as *const u32) };
        let max_slots = (hcsparams1 & 0xFF) as u8;
        let max_ports = ((hcsparams1 >> 24) & 0xFF) as u8;

        let hcsparams2 = unsafe { core::ptr::read_volatile((mmio_virt + 0x08) as *const u32) };
        let max_scratchpad_buffers =
            (((hcsparams2 >> 21) & 0x1f) | (((hcsparams2 >> 27) & 0x1f) << 5)) as usize;

        let hccparams1 = unsafe { core::ptr::read_volatile((mmio_virt + 0x10) as *const u32) };
        let ctx_size = if (hccparams1 & (1 << 2)) != 0 { 64 } else { 32 };

        log::debug!(
            "xHCI initialized: MaxSlots={}, MaxPorts={}, ScratchpadBuffers={}, CtxSize={}",
            max_slots,
            max_ports,
            max_scratchpad_buffers,
            ctx_size
        );

        unsafe {
            let usbcmd_ptr = operational_offset as *mut u32;
            let usbsts_ptr = (operational_offset + 0x04) as *const u32;

            let mut usbcmd = core::ptr::read_volatile(usbcmd_ptr);
            usbcmd &= !1; // Stop controller
            core::ptr::write_volatile(usbcmd_ptr, usbcmd);

            let mut timeout = 100_000;
            while (core::ptr::read_volatile(usbsts_ptr) & 1) == 0 && timeout > 0 {
                core::hint::spin_loop();
                timeout -= 1;
            }

            usbcmd = core::ptr::read_volatile(usbcmd_ptr);
            usbcmd |= 1 << 1; // HCRST
            core::ptr::write_volatile(usbcmd_ptr, usbcmd);

            timeout = 100_000;
            while (core::ptr::read_volatile(usbcmd_ptr) & (1 << 1)) != 0 && timeout > 0 {
                core::hint::spin_loop();
                timeout -= 1;
            }

            timeout = 100_000;
            while (core::ptr::read_volatile(usbsts_ptr) & (1 << 11)) != 0 && timeout > 0 {
                core::hint::spin_loop();
                timeout -= 1;
            }
        }

        unsafe {
            let config_ptr = (operational_offset + 0x38) as *mut u32;
            core::ptr::write_volatile(config_ptr, max_slots as u32);
        }

        let mut sp_arr_phys = 0;
        if max_scratchpad_buffers > 0 {
            let array_bytes = max_scratchpad_buffers * 8;
            let array_pages = array_bytes.div_ceil(4096);
            let (arr_p, arr_v) = unsafe {
                api::memory()
                    .dma_alloc(array_pages)
                    .ok_or(PciError::InitFailed)?
            };
            sp_arr_phys = arr_p;

            for i in 0..max_scratchpad_buffers {
                let (buf_p, _) = unsafe { api::memory().dma_alloc(1).ok_or(PciError::InitFailed)? };
                unsafe {
                    core::ptr::write_volatile((arr_v + i * 8) as *mut u64, buf_p as u64);
                }
            }
        }

        let (dcbaa_phys, dcbaa_virt) =
            unsafe { api::memory().dma_alloc(1).ok_or(PciError::InitFailed)? };
        if max_scratchpad_buffers > 0 {
            unsafe {
                core::ptr::write_volatile(dcbaa_virt as *mut u64, sp_arr_phys as u64);
            }
        }
        write_reg64(operational_offset + 0x30, dcbaa_phys as u64);

        let (cmd_phys, cmd_virt) =
            unsafe { api::memory().dma_alloc(1).ok_or(PciError::InitFailed)? };
        let mut cmd_ring = Ring::new(cmd_phys, cmd_virt, 256);
        write_reg64(operational_offset + 0x18, (cmd_phys as u64) | 1);

        let (ev_phys, ev_virt) = unsafe { api::memory().dma_alloc(1).ok_or(PciError::InitFailed)? };
        let (erst_phys, erst_virt) =
            unsafe { api::memory().dma_alloc(1).ok_or(PciError::InitFailed)? };
        let erst_entry = erst_virt as *mut ErstEntry;
        unsafe {
            core::ptr::write_volatile(
                erst_entry,
                ErstEntry {
                    ring_segment_base_address: ev_phys as u64,
                    ring_segment_size: 256,
                    reserved: 0,
                },
            );
        }

        let int0_base = runtime_offset + 0x20;
        let erstsz_ptr = (int0_base + 0x08) as *mut u32;
        let iman_ptr = int0_base as *mut u32;

        unsafe {
            core::ptr::write_volatile(erstsz_ptr, 1);
        }
        write_reg64(int0_base + 0x10, erst_phys as u64);
        write_reg64(int0_base + 0x18, (ev_phys as u64) | (1 << 3));
        unsafe {
            core::ptr::write_volatile(iman_ptr, core::ptr::read_volatile(iman_ptr) | 0x2);
        }

        let mut event_ring = EventRing {
            phys: ev_phys,
            virt: ev_virt,
            len: 256,
            dequeue: 0,
            ecs: 1,
            erdp_reg: int0_base + 0x18,
        };

        unsafe {
            let usbcmd_ptr = operational_offset as *mut u32;
            let usbsts_ptr = (operational_offset + 0x04) as *const u32;
            let mut usbcmd = core::ptr::read_volatile(usbcmd_ptr);
            usbcmd |= 1 | (1 << 2); // Run/Stop = 1, INTE = 1
            core::ptr::write_volatile(usbcmd_ptr, usbcmd);

            let mut timeout = 100_000;
            while (core::ptr::read_volatile(usbsts_ptr) & 1) != 0 && timeout > 0 {
                core::hint::spin_loop();
                timeout -= 1;
            }
        }

        for port_idx in 0..max_ports {
            let portsc_ptr = (operational_offset + 0x400 + port_idx as usize * 0x10) as *mut u32;
            let portsc = unsafe { core::ptr::read_volatile(portsc_ptr) };
            let rw_mask = 0x0E00_0E0E;
            let new_sc = (portsc & rw_mask) | (1 << 9); // PP (bit 9) = 1
            unsafe {
                core::ptr::write_volatile(portsc_ptr, new_sc);
            }
        }

        // Settle delay
        for _ in 0..100_000 {
            core::hint::spin_loop();
        }

        let mut keyboards = Vec::new();

        for port_idx in 0..max_ports {
            let portsc_ptr = (operational_offset + 0x400 + port_idx as usize * 0x10) as *mut u32;
            let portsc = unsafe { core::ptr::read_volatile(portsc_ptr) };
            if (portsc & 1) == 0 {
                continue; // No device connected
            }

            log::debug!("xHCI: Device detected on port {}", port_idx + 1);

            // Reset port
            let rw_mask = 0x0E00_0E0E;
            let pr_sc = (portsc & rw_mask) | (1 << 4);
            unsafe {
                core::ptr::write_volatile(portsc_ptr, pr_sc);
            }

            let mut timeout = 200_000;
            while timeout > 0 {
                let sc = unsafe { core::ptr::read_volatile(portsc_ptr) };
                if (sc & (1 << 4)) == 0 && ((sc & (1 << 1)) != 0 || (sc & (1 << 21)) != 0) {
                    break;
                }
                core::hint::spin_loop();
                timeout -= 1;
            }

            let sc = unsafe { core::ptr::read_volatile(portsc_ptr) };
            let clear_sc = (sc & rw_mask) | (1 << 21) | (1 << 17);
            unsafe {
                core::ptr::write_volatile(portsc_ptr, clear_sc);
            }

            let sc_after = unsafe { core::ptr::read_volatile(portsc_ptr) };
            let port_speed = ((sc_after >> 10) & 0xF) as u32;
            log::debug!(
                "xHCI: Port {} reset complete, speed = {}",
                port_idx + 1,
                port_speed
            );

            // Enable Slot
            let ev = send_command(
                &mut cmd_ring,
                doorbell_offset,
                &mut event_ring,
                Trb {
                    parameter: 0,
                    status: 0,
                    control: 9 << 10,
                },
            )
            .map_err(|_| PciError::InitFailed)?;
            let slot_id = ((ev.control >> 24) & 0xFF) as u8;
            log::info!(
                "xHCI: Allocated Slot ID {} for port {}",
                slot_id,
                port_idx + 1
            );

            // Allocate Input & Output Contexts
            let (input_phys, input_virt) =
                unsafe { api::memory().dma_alloc(1).ok_or(PciError::InitFailed)? };
            let (output_phys, _) =
                unsafe { api::memory().dma_alloc(1).ok_or(PciError::InitFailed)? };

            // Allocate EP0 Ring
            let (ep0_phys, ep0_virt) =
                unsafe { api::memory().dma_alloc(1).ok_or(PciError::InitFailed)? };
            let mut ep0_ring = Ring::new(ep0_phys, ep0_virt, 256);

            // MaxPacketSize: 64 for High-speed (speed 3), 8 for Full/Low-speed
            let ep0_mps = if port_speed == 3 { 64u32 } else { 8u32 };

            // Populate Input Context for Address Device
            write_ctx_u32(input_virt, ctx_size, 0, 0, 0);
            write_ctx_u32(input_virt, ctx_size, 0, 1, (1 << 0) | (1 << 1)); // Slot + EP0

            let sc_dw0 = (port_speed << 20) | (1 << 27); // Entries = 1
            write_ctx_u32(input_virt, ctx_size, 1, 0, sc_dw0);
            let sc_dw1 = ((port_idx + 1) as u32) << 16;
            write_ctx_u32(input_virt, ctx_size, 1, 1, sc_dw1);

            write_ctx_u32(input_virt, ctx_size, 2, 0, 0);
            let ep0_dw1 = (3 << 1) | (4 << 3) | (ep0_mps << 16); // CErr=3, Control, MaxPacketSize
            write_ctx_u32(input_virt, ctx_size, 2, 1, ep0_dw1);
            let ep0_dw2 = ((ep0_ring.phys as u32) & !0xF) | 1;
            write_ctx_u32(input_virt, ctx_size, 2, 2, ep0_dw2);
            let ep0_dw3 = ((ep0_ring.phys as u64) >> 32) as u32;
            write_ctx_u32(input_virt, ctx_size, 2, 3, ep0_dw3);
            write_ctx_u32(input_virt, ctx_size, 2, 4, 8);

            // Assign Output Context in DCBAA
            let dcbaa_slot_ptr = (dcbaa_virt + slot_id as usize * 8) as *mut u64;
            unsafe {
                core::ptr::write_volatile(dcbaa_slot_ptr, output_phys as u64);
            }

            // Address Device
            send_command(
                &mut cmd_ring,
                doorbell_offset,
                &mut event_ring,
                Trb {
                    parameter: input_phys as u64,
                    status: 0,
                    control: (11 << 10) | ((slot_id as u32) << 24),
                },
            )
            .map_err(|_| PciError::InitFailed)?;
            log::debug!("xHCI: Slot {} addressed successfully", slot_id);

            // Allocate EP1 IN Ring
            let (ep1_phys, ep1_virt) =
                unsafe { api::memory().dma_alloc(1).ok_or(PciError::InitFailed)? };
            let mut ep1_ring = Ring::new(ep1_phys, ep1_virt, 256);

            // Clear Input Context for Configure Endpoint
            unsafe {
                core::ptr::write_bytes(input_virt as *mut u8, 0, 4096);
            }

            write_ctx_u32(input_virt, ctx_size, 0, 0, 0);
            write_ctx_u32(input_virt, ctx_size, 0, 1, (1 << 0) | (1 << 3)); // Slot + EP1 IN (DCI 3)

            let sc_dw0 = (port_speed << 20) | (3 << 27); // Entries = 3
            write_ctx_u32(input_virt, ctx_size, 1, 0, sc_dw0);
            let sc_dw1 = ((port_idx + 1) as u32) << 16;
            write_ctx_u32(input_virt, ctx_size, 1, 1, sc_dw1);

            write_ctx_u32(input_virt, ctx_size, 4, 0, 6 << 16); // Interval = 6
            let ep1_dw1 = (3 << 1) | (7 << 3) | (8 << 16); // CErr=3, Interrupt IN (7), MaxPacketSize=8
            write_ctx_u32(input_virt, ctx_size, 4, 1, ep1_dw1);
            let ep1_dw2 = ((ep1_ring.phys as u32) & !0xF) | 1;
            write_ctx_u32(input_virt, ctx_size, 4, 2, ep1_dw2);
            let ep1_dw3 = ((ep1_ring.phys as u64) >> 32) as u32;
            write_ctx_u32(input_virt, ctx_size, 4, 3, ep1_dw3);
            write_ctx_u32(input_virt, ctx_size, 4, 4, 8 | (8 << 16));

            // Configure Endpoint
            send_command(
                &mut cmd_ring,
                doorbell_offset,
                &mut event_ring,
                Trb {
                    parameter: input_phys as u64,
                    status: 0,
                    control: (12 << 10) | ((slot_id as u32) << 24),
                },
            )
            .map_err(|_| PciError::InitFailed)?;
            log::debug!("xHCI: Slot {} Endpoint 1 IN configured", slot_id);

            // USB Control Requests via EP0
            let _ = ep0_control_transfer(
                &mut ep0_ring,
                doorbell_offset,
                &mut event_ring,
                slot_id,
                0x00,
                0x09,
                1,
                0,
            );

            // SET_PROTOCOL (0 = Boot Protocol)
            let _ = ep0_control_transfer(
                &mut ep0_ring,
                doorbell_offset,
                &mut event_ring,
                slot_id,
                0x21,
                0x0B,
                0,
                0,
            );

            // SET_IDLE (0)
            let _ = ep0_control_transfer(
                &mut ep0_ring,
                doorbell_offset,
                &mut event_ring,
                slot_id,
                0x21,
                0x0A,
                0,
                0,
            );

            // Allocate Report DMA buffer
            let (report_phys, report_virt) =
                unsafe { api::memory().dma_alloc(1).ok_or(PciError::InitFailed)? };

            // Queue first Normal TRB on EP1 IN ring
            let normal_control = (1 << 10) | (1 << 5) | (1 << 2); // Normal TRB, IOC=1, ISP=1
            let _ = ep1_ring.enqueue(Trb {
                parameter: report_phys as u64,
                status: 8,
                control: normal_control,
            });

            ring_doorbell(doorbell_offset, slot_id, 3);
            log::debug!("xHCI: Queued first interrupt transfer for Slot {}", slot_id);

            let device_id = ((port_idx as u32) << 8) | slot_id as u32;
            let descriptor = UsbDeviceDescriptor {
                length: 18,
                descriptor_type: crate::device::usb::DESC_TYPE_DEVICE,
                bcd_usb: 0x0200,
                device_class: 0x03,
                device_subclass: 0x01,
                device_protocol: 0x01,
                max_packet_size0: 8,
                vendor_id: 0x1234,
                product_id: 0x5678,
                bcd_device: 0x0100,
                manufacturer_idx: 0,
                product_idx: 0,
                serial_number_idx: 0,
                num_configurations: 1,
            };

            let handle = Box::new(XhciDeviceHandle::new(slot_id));
            let usb_device = UsbDevice::new(port_idx + 1, descriptor, handle);

            USB_HUB.get().run_mut(|hub| {
                log::debug!("Registering USB device from port {}", port_idx + 1);
                let _ = hub.register_device(device_id, usb_device);
            });

            keyboards.push(KeyboardSlot {
                slot_id,
                device_id,
                ep0_ring,
                ep1_ring,
                report_phys,
                report_virt,
            });
        }

        GLOBAL_XHCI.run_irq(|s| {
            *s = Some(XhciState {
                mmio_virt,
                mmio_len: bar_size,
                operational_offset,
                runtime_offset,
                doorbell_offset,
                cmd_ring,
                event_ring,
                keyboards,
            });
        });

        // Spawn periodic poller task into the kernel scheduler
        crate::scheduler::kernel::KERNEL_SCHEDULER.spawn(poll_xhci_events);

        if let Some(vector) = msi_vector {
            log::info!("xHCI bound with MSI vector: {vector}");
        }

        Ok(())
    }

    fn handle_interrupt(&self, _: &PciDevice) -> InterruptStatus {
        GLOBAL_XHCI.run_irq(|s| {
            if let Some(state) = s.as_mut() {
                // Clear IP in Interrupter 0
                let iman_ptr = (state.runtime_offset + 0x20) as *mut u32;
                unsafe {
                    let iman = core::ptr::read_volatile(iman_ptr);
                    core::ptr::write_volatile(iman_ptr, iman | 0x1);
                }

                // Clear EINT in USBSTS
                let usbsts_ptr = (state.operational_offset + 0x04) as *mut u32;
                unsafe {
                    core::ptr::write_volatile(usbsts_ptr, 1 << 3);
                }

                state.poll_events();
            }
        });

        InterruptStatus::Handled
    }

    fn destroy(&self, _: &PciDevice) -> Result<(), PciError> {
        GLOBAL_XHCI.run_irq(|s| *s = None);
        Ok(())
    }
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
        if data.is_empty() {
            GLOBAL_XHCI.run_irq(|s| {
                if let Some(state) = s.as_mut() {
                    let db_offset = state.doorbell_offset;
                    if let Some(kbd) = state
                        .keyboards
                        .iter_mut()
                        .find(|k| k.slot_id == self.slot_id)
                    {
                        let _ = ep0_control_transfer(
                            &mut kbd.ep0_ring,
                            db_offset,
                            &mut state.event_ring,
                            self.slot_id,
                            request_type,
                            request,
                            value,
                            index,
                        );
                    }
                }
            });
        }

        Ok(data.len())
    }

    fn submit_interrupt_transfer(
        &self,
        _endpoint: u8,
        buffer: &mut [u8],
    ) -> Result<usize, UsbError> {
        Ok(buffer.len())
    }
}
