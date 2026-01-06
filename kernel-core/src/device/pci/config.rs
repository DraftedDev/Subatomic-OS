use core::ptr::{read_volatile, write_volatile};
use pci_types::{ConfigRegionAccess, PciAddress};

/// PCIe ECAM (MMCONFIG) configuration space access.
///
/// This provides memory-mapped access to PCI configuration space
/// as defined by the PCI Express specification.
#[derive(Debug, Copy, Clone)]
pub struct PciConfig {
    base_address: usize,
}

impl PciConfig {
    /// Create a new ECAM config accessor.
    ///
    /// `base_address` must point to the mapped ECAM region.
    pub const fn new(base_address: usize) -> Self {
        Self { base_address }
    }

    #[inline(always)]
    fn ecam_address(&self, address: PciAddress, offset: u16) -> *mut u32 {
        let bus = address.bus() as usize;
        let device = address.device() as usize;
        let function = address.function() as usize;

        let addr = self.base_address
            + (bus << 20)
            + (device << 15)
            + (function << 12)
            + (offset as usize & 0xFFC);

        addr as *mut u32
    }
}

impl ConfigRegionAccess for PciConfig {
    unsafe fn read(&self, address: PciAddress, offset: u16) -> u32 {
        let ptr = self.ecam_address(address, offset);
        unsafe { read_volatile(ptr) }
    }

    unsafe fn write(&self, address: PciAddress, offset: u16, value: u32) {
        let ptr = self.ecam_address(address, offset);
        unsafe {
            write_volatile(ptr, value);
        }
    }
}
