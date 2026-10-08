use crate::api;
use crate::collections::FastMap;
use crate::device::pci::caps::PciCapabilities;
use crate::device::pci::classes::Class;
use crate::device::pci::config::PciConfig;
use crate::device::pci::drivers::virtio;
use crate::device::pci::error::PciError;
use crate::device::{Device, DeviceHub};
use crate::sync::init::InitData;
use crate::sync::rwlock::RwLock;
use alloc::boxed::Box;
use alloc::vec::Vec;
use pci_types::{
    Bar, CommandRegister, ConfigRegionAccess, DeviceId, DeviceRevision, HeaderType, Interface,
    PciAddress, PciHeader, VendorId,
};

/// Contains PCI device capabilities.
pub mod caps;

/// Contains classes and subclasses of PCI devices.
pub mod classes;

/// Contains the [PciConfig] type.
pub mod config;

/// Contains the [PciError] type.
pub mod error;

/// Contains built-in PCI drivers.
pub mod drivers;

/// A hardware abstraction layer for PCI devices.
pub mod hal;

/// The global [PciDeviceHub].
pub static PCI_HUB: InitData<RwLock<PciDeviceHub>> = InitData::uninit();

/// Initialize the global [PCI_HUB] with the given ECAM base address.
///
/// # Safety
///
/// This function is unsafe, because the caller must guarantee
/// that this is called before any [PciDeviceHub] operations and only once.
pub unsafe fn init<'a>(ecam_base: usize) -> &'a RwLock<PciDeviceHub> {
    unsafe { PCI_HUB.init(RwLock::new(PciDeviceHub::new(ecam_base))) }
}

/// Add built-in drivers to the global [PCI_HUB].
pub fn add_builtin_drivers() -> Result<(), PciError> {
    PCI_HUB
        .get()
        .run_mut(|hub| hub.register(Box::new(virtio::gpu::VirtioGpuDriver::new())))
}

/// The device hub to control all PCI devices.
pub struct PciDeviceHub {
    devices: FastMap<u32, PciDevice>,
    drivers: FastMap<&'static str, Box<dyn PciDriver>>,
    driver_devices: FastMap<&'static str, Vec<u32>>,
    config: PciConfig,
    vector_map: [Vec<InterruptBinding>; 256],
    allocated_vectors: [bool; 256],
}

impl PciDeviceHub {
    /// Create a new [PciDeviceHub] with the given ECAM base address.
    pub fn new(ecam_base: usize) -> Self {
        Self {
            devices: FastMap::default(),
            drivers: FastMap::default(),
            driver_devices: FastMap::default(),
            config: PciConfig::new(ecam_base),
            vector_map: core::array::from_fn(|_| Vec::new()),
            allocated_vectors: [false; 256],
        }
    }

    /// Add a driver to the hub.
    ///
    /// The driver may or may not be used, depending on device availability.
    pub fn add_driver(&mut self, driver: impl PciDriver) {
        self.drivers.insert(driver.name(), Box::new(driver));
    }

    /// Get all registered driver names.
    pub fn get_drivers(&self) -> Vec<&'static str> {
        self.drivers.keys().copied().collect()
    }

    /// Get all devices that the given driver operate on.
    pub fn devices_of_driver(&self, driver_name: &'static str) -> Option<Vec<&PciDevice>> {
        self.driver_devices.get(driver_name).map(|device_ids| {
            device_ids
                .iter()
                .filter_map(|id| self.devices.get(id))
                .collect()
        })
    }

    /// Get all device IDs that the given driver operate on.
    pub fn get_driver_devices(&self, driver: &'static str) -> Option<&Vec<u32>> {
        self.driver_devices.get(driver)
    }

    /// Allocates an unused vector and configures the target device for MSI interrupts.
    pub fn attach_msi_interrupt(
        &mut self,
        device_id: u32,
        driver_name: &'static str,
        target_apic_id: u8,
    ) -> Result<u8, PciError> {
        let device = self
            .devices
            .get(&device_id)
            .ok_or(PciError::DeviceNotFound)?;

        // Find a free vector between 32 and 255 (0..31 are x86 CPU exceptions)
        let vector = (32..=255)
            .find(|&v| !self.allocated_vectors[v])
            .ok_or(PciError::DriverHubFull)? as u8;

        // Try programming MSI on hardware
        if !device.enable_msi(vector, target_apic_id) {
            return Err(PciError::MsiUnsupported);
        }

        // Lock in vector allocation & save binding
        self.allocated_vectors[vector as usize] = true;
        self.vector_map[vector as usize].push(InterruptBinding {
            driver_name,
            device_id,
        });

        Ok(vector)
    }

    /// Entry point triggered by CPU IDT stubs when a PCI interrupt vector fires.
    pub fn dispatch_interrupt(&self, vector: u8) {
        let bindings = &self.vector_map[vector as usize];

        for binding in bindings {
            if let (Some(driver), Some(device)) = (
                self.drivers.get(binding.driver_name),
                self.devices.get(&binding.device_id),
            ) && driver.handle_interrupt(device) == InterruptStatus::Handled
            {
                break;
            }
        }

        unsafe {
            api::interrupts().end_of_interrupt();
        }
    }

    /// Allocate a device vector.
    pub fn allocate_vector(&mut self) -> Option<u8> {
        (32..=255).find(|&v| !self.is_vector_reserved(v) && !self.allocated_vectors[v as usize])
    }

    fn is_vector_reserved(&self, vector: u8) -> bool {
        matches!(vector, 0..=33 | 51 | 63)
    }

    /// Helper: enumerates one function of a device
    fn enumerate_function(
        &mut self,
        segment: u16,
        bus: u8,
        device: u8,
        function: u8,
    ) -> Result<(), PciError> {
        let addr = PciAddress::new(segment, bus, device, function);

        // Read vendor ID to check if the device exists
        let vendor = unsafe { self.config.read(addr, 0x00) & 0xFFFF } as u16;
        if vendor == 0xFFFF {
            return Ok(()); // no device here
        }

        let dev = PciDevice::new(addr, self.config);

        let device_id = ((segment as u32) << 24)
            | ((bus as u32) << 16)
            | ((device as u32) << 11)
            | ((function as u32) << 8);
        self.devices.insert(device_id, dev);

        Ok(())
    }

    /// Helper: enumerates a single device (may have multiple functions)
    fn enumerate_device(&mut self, segment: u16, bus: u8, device: u8) -> Result<(), PciError> {
        self.enumerate_function(segment, bus, device, 0)?;

        let header_type = unsafe {
            self.config
                .read(PciAddress::new(segment, bus, device, 0), 0x0C)
                >> 16
                & 0xFF
        };
        if (header_type & 0x80) != 0 {
            // Enumerate functions 1..7
            for func in 1..8 {
                self.enumerate_function(segment, bus, device, func)?;
            }
        }

        Ok(())
    }

    /// Helper: enumerates a single bus
    fn enumerate_bus(&mut self, segment: u16, bus: u8) -> Result<(), PciError> {
        for device in 0..32 {
            self.enumerate_device(segment, bus, device)?;
        }
        Ok(())
    }
}

impl DeviceHub for PciDeviceHub {
    type Device = PciDevice;
    type DeviceId = u32;
    type Driver = Box<dyn PciDriver>;
    type DriverId = &'static str;
    type Error = PciError;

    fn init(&mut self) -> Result<(), Self::Error> {
        // Modern PCI allows 0 to =255 buses, usually only segment 0 exists
        let segments = [0u16]; // TODO: extend if multiple segments
        for segment in segments {
            for bus in 0..=255 {
                self.enumerate_bus(segment, bus)?;
            }
        }

        Ok(())
    }

    fn register(&mut self, driver: Self::Driver) -> Result<(), Self::Error> {
        if self.drivers.contains_key(driver.name()) {
            return Err(PciError::DriverAlreadyRegistered);
        }

        let mut bound_device_ids = Vec::new();
        let device_ids: Vec<u32> = self.devices.keys().copied().collect();

        for id in device_ids {
            let device = self.devices.get(&id).ok_or(PciError::DeviceNotFound)?;

            if driver.should_bind(device) {
                log::info!(
                    "Binding device {} to driver {}",
                    device.addr(),
                    driver.name()
                );

                let msi_vector = if let Some(apic_id) = driver.should_attach_msi(device) {
                    Some(
                        // Allocate a vector and configure the device for MSI
                        {
                            let driver_name = driver.name();
                            let device = self.devices.get(&id).ok_or(PciError::DeviceNotFound)?;
                            let vector = (32..=255)
                                .find(|&v| !self.allocated_vectors[v])
                                .ok_or(PciError::DriverHubFull)?
                                as u8;

                            if !device.enable_msi(vector, apic_id) {
                                return Err(PciError::MsiUnsupported);
                            }

                            self.allocated_vectors[vector as usize] = true;
                            self.vector_map[vector as usize].push(InterruptBinding {
                                driver_name,
                                device_id: id,
                            });

                            Result::<u8, PciError>::Ok(vector)
                        }?,
                    )
                } else {
                    None
                };

                driver.init(device, msi_vector)?;
                bound_device_ids.push(id);
            }
        }

        self.driver_devices.insert(driver.name(), bound_device_ids);
        self.drivers.insert(driver.name(), driver);

        Ok(())
    }

    fn unregister(&mut self, driver_name: &'static str) -> Result<Self::Driver, Self::Error> {
        let driver = self
            .drivers
            .remove(driver_name)
            .ok_or(PciError::DriverNotFound)?;

        let devices = self
            .driver_devices
            .remove(driver.name())
            .ok_or(PciError::DriverNotFound)?;

        for id in devices {
            if let Ok(device) = self.get(id) {
                driver.destroy(device)?;
            }

            // Clean up any MSI vector bindings associated with this device
            for (vector, bindings) in self.vector_map.iter_mut().enumerate() {
                bindings.retain(|b| {
                    let is_match = b.device_id == id && b.driver_name == driver_name;
                    if is_match {
                        self.allocated_vectors[vector] = false; // Release vector back to pool
                    }
                    !is_match
                });
            }
        }

        Ok(driver)
    }

    fn devices(&self) -> Vec<Self::DeviceId> {
        self.devices.keys().copied().collect()
    }

    fn get(&self, id: Self::DeviceId) -> Result<&Self::Device, Self::Error> {
        self.devices.get(&id).ok_or(PciError::DeviceNotFound)
    }

    fn get_driver(&self, driver: Self::DriverId) -> Result<&Self::Driver, Self::Error> {
        self.drivers.get(driver).ok_or(PciError::DriverNotFound)
    }
}

/// A PCI device.
pub struct PciDevice {
    config: PciConfig,
    addr: PciAddress,
    header: PciHeader,
    header_type: HeaderType,
    id: (VendorId, DeviceId),
    command: CommandRegister,
    class: Class,
    interface: Interface,
    revision: DeviceRevision,
    capabilities: PciCapabilities,
}

impl PciDevice {
    /// Create a new [PciDevice] with the given [PciAddress] and [PciConfig].
    pub fn new(addr: PciAddress, config: PciConfig) -> Self {
        let header = PciHeader::new(addr);
        let id = header.id(config);
        let command = header.command(config);
        let header_type = header.header_type(config);
        let (revision, base, sub, interface) = header.revision_and_class(config);

        Self {
            config,
            addr,
            header,
            header_type,
            id,
            command,
            class: Class::from_u8(base, sub),
            interface,
            revision,
            capabilities: PciCapabilities::new(&config, addr),
        }
    }

    /// Returns if the device has multiple functions.
    pub fn has_multiple_functions(&self) -> bool {
        self.header.has_multiple_functions(self.config)
    }

    /// Returns the PCI device address.
    pub fn addr(&self) -> PciAddress {
        self.addr
    }

    /// Returns the PCI device header.
    pub fn header(&self) -> &PciHeader {
        &self.header
    }

    /// Returns the PCI device header type.
    pub fn header_type(&self) -> HeaderType {
        self.header_type
    }

    /// Returns the PCI device vendor and device ID.
    pub fn id(&self) -> (VendorId, DeviceId) {
        self.id
    }

    /// Returns the PCI device command register.
    pub fn command(&self) -> CommandRegister {
        self.command
    }

    /// Returns the PCI device class.
    pub fn class(&self) -> Class {
        self.class
    }

    /// Returns the PCI device interface.
    pub fn interface(&self) -> Interface {
        self.interface
    }

    /// Returns the PCI device revision.
    pub fn revision(&self) -> DeviceRevision {
        self.revision
    }

    /// Returns the PCI device capabilities.
    pub fn capabilities(&self) -> &PciCapabilities {
        &self.capabilities
    }

    /// Returns the PCI device configuration space.
    pub fn config(&self) -> PciConfig {
        self.config
    }

    /// Enables bus mastering for this PCI device.
    pub fn enable_bus_mastering(&self) {
        const COMMAND_OFFSET: u16 = 0x04;
        const BUS_MASTER_BIT: u32 = 1 << 2;
        const MEMORY_SPACE_BIT: u32 = 1 << 1;

        let mut cmd = unsafe { self.config.read(self.addr, COMMAND_OFFSET) };

        if (cmd & BUS_MASTER_BIT) == 0 {
            cmd |= BUS_MASTER_BIT | MEMORY_SPACE_BIT;
            unsafe { self.config.write(self.addr, COMMAND_OFFSET, cmd) };
        }
    }

    /// Returns the BAR at the given index.
    pub fn bar(&self, index: usize) -> Option<Bar> {
        if index > 5 {
            return None;
        }

        const BAR0_OFFSET: u16 = 0x10;
        let offset = BAR0_OFFSET + (index as u16) * 4;

        // Read original BAR value
        let original = unsafe { self.config.read(self.addr, offset) };
        if original == 0 {
            return None; // BAR not implemented
        }

        // Check if I/O space
        if (original & 0x1) != 0 {
            // I/O BAR
            let port_addr = original & 0xFFFFFFFC;
            Some(Bar::Io { port: port_addr })
        } else {
            // Memory BAR
            let prefetchable = (original & 0x8) != 0;
            let bar_type = (original >> 1) & 0x3;

            match bar_type {
                0 => {
                    // 32-bit MMIO
                    // Determine size
                    unsafe { self.config.write(self.addr, offset, 0xFFFF_FFFF) };
                    let size = !(unsafe { self.config.read(self.addr, offset) } & 0xFFFF_FFF0) + 1;
                    unsafe { self.config.write(self.addr, offset, original) }; // restore
                    Some(Bar::Memory32 {
                        address: original & 0xFFFF_FFF0,
                        size,
                        prefetchable,
                    })
                }
                2 => {
                    // 64-bit MMIO (uses next BAR too)
                    let low = original & 0xFFFF_FFF0;
                    let next_offset = offset + 4;
                    let high = unsafe { self.config.read(self.addr, next_offset) };
                    let original_high = high;
                    unsafe { self.config.write(self.addr, offset, 0xFFFF_FFF0) };
                    unsafe { self.config.write(self.addr, next_offset, 0xFFFF_FFFF) };
                    let size_low = unsafe { self.config.read(self.addr, offset) } & 0xFFFF_FFF0;
                    let size_high = unsafe { self.config.read(self.addr, next_offset) };
                    let size = !(u64::from(size_high) << 32 | u64::from(size_low)) + 1;
                    unsafe { self.config.write(self.addr, offset, original) };
                    unsafe { self.config.write(self.addr, next_offset, original_high) };

                    Some(Bar::Memory64 {
                        address: (u64::from(high) << 32) | u64::from(low),
                        size,
                        prefetchable,
                    })
                }
                _ => None, // reserved / unsupported
            }
        }
    }

    /// Configure the PCI device for MSI.
    pub fn enable_msi(&self, vector: u8, apic_id: u8) -> bool {
        // 1. Try standard MSI (Cap ID 0x05)
        if let Some(msi_offset) = self.find_capability_offset(0x05) {
            return self.configure_standard_msi(msi_offset, vector, apic_id);
        }

        // 2. Try MSI-X (Cap ID 0x11)
        if let Some(msix_offset) = self.find_capability_offset(0x11) {
            return self.configure_msix(msix_offset, vector, apic_id);
        }

        false
    }

    fn configure_standard_msi(&self, msi_offset: u16, vector: u8, apic_id: u8) -> bool {
        let msg_ctrl = unsafe { self.config.read(self.addr, msi_offset + 2) } as u16;
        let msg_addr: u32 = 0xFEE0_0000 | ((apic_id as u32) << 12);

        unsafe {
            self.config.write(self.addr, msi_offset + 4, msg_addr);
        }

        let is_64bit = (msg_ctrl & (1 << 7)) != 0;
        let data_offset = if is_64bit {
            msi_offset + 12
        } else {
            msi_offset + 8
        };

        unsafe {
            self.config.write(self.addr, data_offset, vector as u32);
            self.config
                .write(self.addr, msi_offset + 2, (msg_ctrl | 0x0001) as u32);
        }

        true
    }

    fn configure_msix(&self, msix_offset: u16, vector: u8, apic_id: u8) -> bool {
        let msg_ctrl = unsafe { self.config.read(self.addr, msix_offset + 2) } as u16;
        let table_info = unsafe { self.config.read(self.addr, msix_offset + 4) };

        let table_bar_idx = (table_info & 0x7) as usize;
        let table_offset = (table_info & !0x7) as usize;

        // Get BAR base address for MSI-X table
        let bar_base = match self.bar(table_bar_idx) {
            Some(Bar::Memory32 { address, .. }) => address as usize,
            Some(Bar::Memory64 { address, .. }) => address as usize,
            _ => return false,
        };

        // Map table entry in kernel space
        let table_entry_addr = bar_base + table_offset;
        let virt_addr = unsafe { crate::api::memory().map_to(table_entry_addr, true, false) };

        let entry_ptr = virt_addr as *mut u32;

        // Write Msg Addr, Msg Data, and Vector Control (0 = Unmasked)
        let msg_addr: u32 = 0xFEE0_0000 | ((apic_id as u32) << 12);
        unsafe {
            core::ptr::write_volatile(entry_ptr.add(0), msg_addr); // Msg Addr Lower
            core::ptr::write_volatile(entry_ptr.add(1), 0); // Msg Addr Upper
            core::ptr::write_volatile(entry_ptr.add(2), vector as u32); // Msg Data
            core::ptr::write_volatile(entry_ptr.add(3), 0); // Vector Control (Unmask)
        }

        // Enable MSI-X in Control Register (Bit 15 = Enable)
        let updated_ctrl = (msg_ctrl | (1 << 15)) as u32;
        unsafe {
            self.config.write(self.addr, msix_offset + 2, updated_ctrl);
        }

        true
    }

    fn find_capability_offset(&self, cap_id: u8) -> Option<u16> {
        // Read Status register (offset 0x06) to verify Capabilities List bit (bit 4) is set
        let status = unsafe { self.config.read(self.addr, 0x06) >> 16 } as u16;
        if (status & (1 << 4)) == 0 {
            return None;
        }

        // Read Capabilities Pointer (offset 0x34)
        let mut cap_ptr = (unsafe { self.config.read(self.addr, 0x34) } & 0xFF) as u16;

        while cap_ptr != 0 {
            let cap_header = unsafe { self.config.read(self.addr, cap_ptr) };
            let current_id = (cap_header & 0xFF) as u8;

            if current_id == cap_id {
                return Some(cap_ptr);
            }

            // Move to next capability pointer in line
            cap_ptr = ((cap_header >> 8) & 0xFF) as u16;
        }

        None
    }
}

impl Device for PciDevice {
    type DeviceId = u32;
    type Error = PciError;
}

/// A trait to define PCI device drivers.
///
/// Drivers should implement any message signaling and other functions by themselves.
pub trait PciDriver: Send + Sync + 'static {
    /// A unique name for the driver.
    fn name(&self) -> &'static str;

    /// Returns if this driver should be bound to the given device.
    ///
    /// This is where drivers should check device capabilities and other properties.
    fn should_bind(&self, device: &PciDevice) -> bool;

    /// Determines if the device should be attached with an MSI interrupt.
    ///
    /// The returned value should be the local APIC ID.
    fn should_attach_msi(&self, device: &PciDevice) -> Option<u8>;

    /// Initialize the device driver.
    ///
    /// If [should_attach_msi] returns [Some],
    /// the `msi_vector` parameter will contain the allocated vector for the device.
    fn init(&self, device: &PciDevice, msi_vector: Option<u8>) -> Result<(), PciError>;

    /// Handle a device interrupt.
    fn handle_interrupt(&self, _: &PciDevice) -> InterruptStatus {
        InterruptStatus::Ignored
    }

    /// Destroys the device driver.
    fn destroy(&self, device: &PciDevice) -> Result<(), PciError>;
}

/// An interrupt status returned [PciDriver::handle_interrupt].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptStatus {
    /// The interrupt originated from this device and was processed.
    Handled,
    /// The interrupt was not generated by this device.
    Ignored,
}

/// A binding for a device interrupt.
pub struct InterruptBinding {
    /// The driver name.
    pub driver_name: &'static str,
    /// The device ID.
    pub device_id: u32,
}
