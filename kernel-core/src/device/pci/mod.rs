// TODO: document
#![allow(missing_docs)]

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

pub mod caps;
pub mod classes;
pub mod config;
pub mod error;

pub static PCI_HUB: InitData<RwLock<PciDeviceHub>> = InitData::uninit();

pub unsafe fn init<'a>(ecam_base: usize) -> &'a RwLock<PciDeviceHub> {
    unsafe { PCI_HUB.init(RwLock::new(PciDeviceHub::new(ecam_base))) }
}

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

    pub fn has_multiple_functions(&self) -> bool {
        self.header.has_multiple_functions(self.config)
    }

    pub fn addr(&self) -> PciAddress {
        self.addr
    }

    pub fn header(&self) -> &PciHeader {
        &self.header
    }

    pub fn header_type(&self) -> HeaderType {
        self.header_type
    }

    pub fn id(&self) -> (VendorId, DeviceId) {
        self.id
    }

    pub fn command(&self) -> CommandRegister {
        self.command
    }

    pub fn class(&self) -> Class {
        self.class
    }

    pub fn interface(&self) -> Interface {
        self.interface
    }

    pub fn revision(&self) -> DeviceRevision {
        self.revision
    }

    pub fn capabilities(&self) -> &PciCapabilities {
        &self.capabilities
    }
}

impl Device for PciDevice {
    type DeviceId = u32;
    type Error = PciError;
}

pub struct PciDeviceHub {
    devices: FastMap<u32, (PciDevice, Box<dyn PciDriver>)>,
    config: PciConfig,
}

impl PciDeviceHub {
    pub fn new(ecam_base: usize) -> Self {
        Self {
            devices: FastMap::default(),
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
        self.devices.insert(device_id, (dev, Box::new(NoopDriver)));

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
        self.devices
            .get(id)
            .ok_or(PciError::DeviceNotFound)
            .map(|(dev, _)| dev)
    }

    fn get_driver(&self, id: &Self::DeviceId) -> Result<&Self::Driver, Self::Error> {
        self.devices
            .get(id)
            .ok_or(PciError::DeviceNotFound)
            .map(|(_, driver)| driver)
    }

    fn install(
        &mut self,
        driver: Self::Driver,
        id: &Self::DeviceId,
    ) -> Result<Self::Driver, Self::Error> {
        let (dev, old_driver) = self.devices.remove(id).ok_or(PciError::DeviceNotFound)?;

        self.devices.insert(*id, (dev, driver));

        Ok(old_driver)
    }
}

pub trait PciDriver: Send + Sync + 'static {
    fn init(&self, device: &PciDevice);
}

struct NoopDriver;

impl PciDriver for NoopDriver {
    fn init(&self, _: &PciDevice) {}
}
