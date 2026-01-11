use crate::collections::FastMap;
use crate::device::pci::caps::PciCapabilities;
use crate::device::pci::classes::Class;
use crate::device::pci::config::PciConfig;
use crate::device::pci::error::PciError;
use crate::device::{Device, DeviceHub};
use crate::sync::init::InitData;
use crate::sync::rwlock::RwLock;
use alloc::boxed::Box;
use alloc::vec::Vec;
use pci_types::{
    CommandRegister, ConfigRegionAccess, DeviceId, DeviceRevision, HeaderType, Interface,
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

/// The global [PciDeviceHub].
pub static PCI_HUB: InitData<RwLock<PciDeviceHub>> = InitData::uninit();

/// Initialize the global [PCI_HUB] with the given ECAM base address.
///
/// # Safety
/// This function is unsafe, because the caller must guarantee
/// that this is called before any [PciDeviceHub] operations and only once.
pub unsafe fn init<'a>(ecam_base: usize) -> &'a RwLock<PciDeviceHub> {
    unsafe { PCI_HUB.init(RwLock::new(PciDeviceHub::new(ecam_base))) }
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
}

impl Device for PciDevice {
    type DeviceId = u32;
    type Error = PciError;
}

/// The device hub to control all PCI devices.
pub struct PciDeviceHub {
    devices: FastMap<u32, PciDevice>,
    drivers: FastMap<&'static str, Box<dyn PciDriver>>,
    config: PciConfig,
}

impl PciDeviceHub {
    /// Create a new [PciDeviceHub] with the given ECAM base address.
    pub fn new(ecam_base: usize) -> Self {
        Self {
            devices: FastMap::default(),
            drivers: FastMap::default(),
            config: PciConfig::new(ecam_base),
        }
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
    type Error = PciError;

    fn init(&mut self) -> Result<(), Self::Error> {
        // Modern PCI allows 0 to =255 buses, usually only segment 0 exists
        let segments = [0u16]; // TODO: extend if multiple segments
        for &segment in &segments {
            for bus in 0..=255 {
                self.enumerate_bus(segment, bus)?;
            }
        }

        Ok(())
    }

    fn devices(&self) -> Vec<Self::DeviceId> {
        self.devices.keys().cloned().collect::<Vec<_>>()
    }

    fn get(&self, id: &Self::DeviceId) -> Result<&Self::Device, Self::Error> {
        self.devices.get(id).ok_or(PciError::DeviceNotFound)
    }

    fn register(&mut self, driver: Self::Driver) -> Result<(), Self::Error> {
        let driver = self
            .drivers
            .insert(driver.name(), driver)
            .ok_or(PciError::DriverAlreadyRegistered)?;

        for device in self.devices.values() {
            if driver.should_bind(device) {
                driver.init(device);
            }
        }

        Ok(())
    }
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

    /// Initialize the device driver.
    fn init(&self, device: &PciDevice);

    /// Destroys the device driver.
    fn destroy(&self, device: &PciDevice);
}
