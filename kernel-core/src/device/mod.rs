use alloc::vec::Vec;

/// The PCI device module.
#[cfg(feature = "pci")]
pub mod pci;

/// A generic device to implement for external devices.
pub trait Device: Send + Sync {
    /// The unique identifier of the device.
    type DeviceId;
    /// The error type that can be produced by read and write operations.
    type Error;
}

/// A global hub for managing devices.
pub trait DeviceHub {
    /// The device this hub manages.
    type Device: Device<DeviceId = Self::DeviceId>;
    /// The device id of the [Self::Device] this hub manages.
    type DeviceId;
    /// The driver type that works with this hub and device type.
    type Driver;
    /// The error type that can be produced by various operations.
    type Error;

    /// Initialize the hub. This should load all present devices.
    fn init(&mut self) -> Result<(), Self::Error>;

    /// Returns a list of loaded devices by their IDs.
    fn devices(&self) -> Vec<Self::DeviceId>;

    /// Get a device by its ID.
    fn get(&self, id: &Self::DeviceId) -> Result<&Self::Device, Self::Error>;

    /// Registers a driver.
    ///
    /// This should check if the driver can be bound to any device.
    ///
    /// Must be called after [DeviceHub::init].
    fn register(&mut self, driver: Self::Driver) -> Result<(), Self::Error>;
}
