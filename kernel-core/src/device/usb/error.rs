use core::{error::Error, fmt::Display};

use crate::device::usb::error::UsbError::NotSupported;

/// Errors occurring within the USB subsystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbError {
    /// An operation is not supported.
    NotSupported,
    /// A device transfer failed.
    TransferFailed,
    /// An operation timed out.
    Timeout,
    /// This is an invalid device.
    InvalidDevice,
    /// The driver is out of memory.
    OutOfMemory,
}

impl Display for UsbError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            NotSupported => write!(f, "Device operation not supported"),
            UsbError::TransferFailed => write!(f, "Device transfer failed"),
            UsbError::Timeout => write!(f, "Operation timed out"),
            UsbError::InvalidDevice => write!(f, "Invalid device found"),
            UsbError::OutOfMemory => write!(f, "Driver out of memory"),
        }
    }
}

impl Error for UsbError {}
