use crate::device::pci::config::PciConfig;
use heapless::Vec;
use pci_types::{ConfigRegionAccess, PciAddress};

const MAX_VENDOR_CAPS: usize = 128;

/// A strongly typed representation of the PCI/PCIe capabilities of a device.
///
/// This struct aggregates the main capabilities a driver would care about.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PciCapabilities {
    /// PCI Power Management Capability
    pub pmc: Option<PowerManagementCap>,
    /// Message Signaled Interrupts Capability (legacy MSI)
    pub msi: Option<MsiCap>,
    /// MSI-X Capability
    pub msix: Option<MsixCap>,
    /// PCIe Capability
    pub pcie: Option<PcieCap>,
    /// Advanced Error Reporting
    pub aer: Option<AerCap>,
    /// Vendor-specific capabilities (raw)
    pub vendor: Vec<VendorCap, MAX_VENDOR_CAPS>,
    /// If the vendor capability-finding loop overflowed [MAX_VENDOR_CAPS].
    pub overflowed: bool,
}

impl PciCapabilities {
    /// Enumerate capabilities from a device using its config space.
    pub fn new(config: &PciConfig, addr: PciAddress) -> Self {
        let mut pmc = None;
        let mut msi = None;
        let mut msix = None; // <-- fixed
        let mut pcie = None;
        let mut aer = None;
        let mut vendor = Vec::new();
        let mut overflowed = false;

        // Legacy PCI capability list (linked list at offset 0x34)
        let mut offset = unsafe { config.read(addr, 0x34) } as u8;
        while offset != 0 && !vendor.is_full() {
            let header = unsafe { config.read(addr, offset as u16) };
            let cap_id = (header & 0xFF) as u8;
            let next = ((header >> 8) & 0xFF) as u8;

            match cap_id {
                0x01 => pmc = Some(PowerManagementCap::from_config(config, addr, offset)),
                0x05 => msi = Some(MsiCap::from_config(config, addr, offset)),
                0x11 => msix = Some(MsixCap::from_config(config, addr, offset as u16)), // MSI-X ID is 0x11
                _ => {
                    vendor
                        .push(VendorCap {
                            offset: offset as u16,
                            id: cap_id as u16,
                        })
                        .unwrap_or_else(|cap| {
                            overflowed = true;
                            log::warn!("Tried to push Legacy Vendor PCI-Device Capability {cap:?}, but maximum of {MAX_VENDOR_CAPS} reached!");
                        });
                }
            }

            if next == 0 {
                break;
            }
            offset = next;
        }

        // PCIe Extended capabilities (starts at offset 0x100)
        let mut offset = 0x100u16;
        loop {
            let header = unsafe { config.read(addr, offset) };
            if header == 0 {
                break;
            }

            let cap_id = (header & 0xFFFF) as u16;
            let next = ((header >> 20) & 0xFFF) as u16;

            match cap_id {
                0x0001 => pcie = Some(PcieCap::from_config(config, addr, offset)),
                0x0007 => aer = Some(AerCap::from_config(config, addr, offset)),
                _ => vendor.push(VendorCap {
                    offset,
                    id: cap_id & 0xFF,
                }).unwrap_or_else(|cap| {
                    overflowed = true;
                    log::warn!("Tried to push Extended Vendor PCI-Device Capability {cap:?}, but maximum of {MAX_VENDOR_CAPS} reached!");
                }),
            }

            if next == 0 || next <= offset {
                break;
            }

            offset = next;
        }

        Self {
            pmc,
            msi,
            msix,
            pcie,
            aer,
            vendor,
            overflowed,
        }
    }
}

/// PCI Power Management Capability
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct PowerManagementCap {
    /// Power management version (0..3)
    pub version: u8,
    /// PME (Power Management Event) support
    pub pme_support: bool,
    /// Device is currently in D1/D2/D3 state
    pub current_state: u8,
}

impl PowerManagementCap {
    fn from_config(config: &PciConfig, addr: PciAddress, offset: u8) -> Self {
        let word = unsafe { config.read(addr, offset as u16) };
        Self {
            version: (word & 0x07) as u8,
            pme_support: (word & 0x8000) != 0,
            current_state: ((word >> 11) & 0x03) as u8,
        }
    }
}

/// PCI Message Signaled Interrupts Capability
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct MsiCap {
    /// True if 64-bit capable
    pub is_64bit: bool,
    /// Number of vectors supported (2^n)
    pub multiple_message_cap: u8,
}

impl MsiCap {
    fn from_config(config: &PciConfig, addr: PciAddress, offset: u8) -> Self {
        let word = unsafe { config.read(addr, offset as u16) };
        Self {
            is_64bit: (word & 0x80) != 0,
            multiple_message_cap: ((word >> 1) & 0x07) as u8,
        }
    }
}

/// PCI MSI-X Capability
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct MsixCap {
    /// Number of MSI-X vectors
    pub table_size: u16,
    /// Table BAR index
    pub table_bir: u8,
    /// Table offset within BAR
    pub table_offset: u32,
}

impl MsixCap {
    fn from_config(config: &PciConfig, addr: PciAddress, offset: u16) -> Self {
        let word0 = unsafe { config.read(addr, offset) };
        let word1 = unsafe { config.read(addr, offset + 4) };
        Self {
            table_size: ((word0 & 0x7FF) + 1) as u16,
            table_bir: (word1 & 0x07) as u8,
            table_offset: word1 & 0xFFFF_FFF8,
        }
    }
}

/// PCI Express Capability
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct PcieCap {
    /// The PCIe version.
    pub version: u8,
    /// The PCIe device type.
    pub device_type: u8,
    /// If the PCIe device slot is implemented.
    pub slot_implemented: bool,
}

impl PcieCap {
    fn from_config(config: &PciConfig, addr: PciAddress, offset: u16) -> Self {
        let reg = unsafe { config.read(addr, offset + 4) };
        Self {
            version: (reg & 0xF) as u8,
            device_type: ((reg >> 4) & 0x1F) as u8,
            slot_implemented: (reg & (1 << 8)) != 0,
        }
    }
}

/// PCIe Advanced Error Reporting
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct AerCap {
    /// Correctable Error Status
    pub ce_status: u32,
    /// Uncorrectable Error Status
    pub ue_status: u32,
}

impl AerCap {
    fn from_config(config: &PciConfig, addr: PciAddress, offset: u16) -> Self {
        Self {
            ce_status: unsafe { config.read(addr, offset + 4) },
            ue_status: unsafe { config.read(addr, offset + 8) },
        }
    }
}

/// Vendor-specific PCI capability
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct VendorCap {
    /// Offset of this capability in config space
    pub offset: u16,
    /// Capability ID
    pub id: u16,
}
