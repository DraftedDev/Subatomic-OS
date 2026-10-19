use crate::device::pci::config::PciConfig;
use pci_types::{ConfigRegionAccess, PciAddress};
use virtio_drivers::transport::pci::bus::{ConfigurationAccess, DeviceFunction};

/// Contains the VirtIO GPU driver.
pub mod gpu;

/// A VirtIO Pci access.
#[derive(Clone, Debug)]
pub struct VirtIoPciAccess {
    config: PciConfig,
    address: PciAddress,
}

impl VirtIoPciAccess {
    /// Creates a new [VirtIoPciAccess].
    pub fn new(config: PciConfig, address: PciAddress) -> Self {
        Self { config, address }
    }
}

impl ConfigurationAccess for VirtIoPciAccess {
    fn read_word(&self, _func: DeviceFunction, offset: u8) -> u32 {
        unsafe { self.config.read(self.address, offset as u16) }
    }

    fn write_word(&mut self, _func: DeviceFunction, offset: u8, data: u32) {
        unsafe { self.config.write(self.address, offset as u16, data) }
    }

    unsafe fn unsafe_clone(&self) -> Self {
        self.clone()
    }
}
