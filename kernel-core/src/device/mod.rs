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
    /// The driver id of the [Self::Driver] this hub manages.
    type DriverId;
    /// The error type that can be produced by various operations.
    type Error;

    /// Initialize the hub. This should load all present devices.
    fn init(&mut self) -> Result<(), Self::Error>;

    /// Register a driver.
    ///
    /// The driver should be registered for every compatible device.
    fn register(&mut self, driver: Self::Driver) -> Result<(), Self::Error>;

    /// Unregister a driver.
    ///
    /// This will unregister and call the driver's destroy function.
    fn unregister(&mut self, driver: Self::DriverId) -> Result<Self::Driver, Self::Error>;

    /// Returns all registered device IDs.
    fn devices(&self) -> Vec<Self::DeviceId>;

    /// Returns a reference to the device with the given ID.
    fn get(&self, id: Self::DeviceId) -> Result<&Self::Device, Self::Error>;

    /// Returns a reference to the driver with the given name.
    fn get_driver(&self, driver: Self::DriverId) -> Result<&Self::Driver, Self::Error>;
}
