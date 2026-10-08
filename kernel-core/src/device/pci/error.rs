use core::fmt::{Display, Formatter};

/// Error type returned by various PCI operations.
#[derive(Debug)]
pub enum PciError {
    /// Initialization failed.
    InitFailed,
    /// The requested device address does not belong to a registered device.
    DeviceNotFound,
    /// The driver is already registered.
    DriverAlreadyRegistered,
    /// The driver is not registered.
    DriverNotFound,
    /// The driver is not ready to be used.
    DriverNotReady,
    /// The driver hub is full and no more devices may be registered.
    DriverHubFull,
    /// The target device does not support MSI.
    MsiUnsupported,
    /// An untyped generic error occurred.
    Generic(anyhow::Error),
}

impl Display for PciError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            PciError::InitFailed => write!(f, "Failed to initialize PCI device hub"),
            PciError::DeviceNotFound => write!(f, "Device not found"),
            PciError::DriverAlreadyRegistered => write!(f, "Driver already registered"),
            PciError::DriverNotFound => write!(f, "Driver not found"),
            PciError::DriverNotReady => write!(f, "Driver not ready"),
            PciError::DriverHubFull => write!(f, "Driver hub is full"),
            PciError::MsiUnsupported => write!(f, "MSI is unsupported on this device"),
            PciError::Generic(err) => write!(f, "Generic error: {}", err),
        }
    }
}

impl From<anyhow::Error> for PciError {
    fn from(err: anyhow::Error) -> Self {
        PciError::Generic(err)
    }
}
