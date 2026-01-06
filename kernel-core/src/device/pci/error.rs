use core::fmt::{Display, Formatter};

/// Error type returned by various PCI operations.
#[derive(Debug)]
pub enum PciError {
    /// Initialization failed.
    InitFailed,
    /// The requested device address does not belong to a registered device.
    DeviceNotFound,
}

impl Display for PciError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            PciError::InitFailed => write!(f, "Failed to initialize PCI device hub"),
            PciError::DeviceNotFound => write!(f, "Device not found"),
        }
    }
}
