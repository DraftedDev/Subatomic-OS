use crate::collections::FastMap;
use crate::device::DeviceHub;
use crate::drivers::usb::keyboard::UsbKeyboardDriver;
use crate::sync::init::InitData;
use crate::sync::rwlock::RwLock;
use alloc::boxed::Box;
use alloc::vec::Vec;

/// Contains the [UsbError] type.
pub mod error;

pub use error::UsbError;

/// USB descriptor device type.
pub const DESC_TYPE_DEVICE: u8 = 0x01;

/// USB descriptor configuration type.
pub const DESC_TYPE_CONFIGURATION: u8 = 0x02;

/// The global [UsbDeviceHub].
pub static USB_HUB: InitData<RwLock<UsbDeviceHub>> = InitData::uninit();

/// Initialize the global [UsbDeviceHub].
///
/// # Safety
///
/// This function is unsafe, because the caller must guarantee
/// that this is called before any [UsbDeviceHub] operations and only once.
///
/// Also make sure that the PCI device hub is initialized after this hub.
pub unsafe fn init<'a>() -> &'a mut RwLock<UsbDeviceHub> {
    USB_HUB.init(RwLock::new(UsbDeviceHub::new()))
}

/// Add built-in drivers to the global [USB_HUB].
pub fn add_builtin_drivers() -> Result<(), UsbError> {
    USB_HUB.get().run_mut(|hub| {
        hub.register(Box::new(UsbKeyboardDriver::new()))?;

        Ok(())
    })
}

/// The central hub to control all USB devices and drivers.
pub struct UsbDeviceHub {
    devices: FastMap<u32, UsbDevice>,
    drivers: FastMap<&'static str, Box<dyn UsbDriver>>,
    driver_devices: FastMap<&'static str, Vec<u32>>,
}

impl UsbDeviceHub {
    /// Create a new [UsbDeviceHub].
    pub fn new() -> Self {
        Self {
            devices: FastMap::default(),
            drivers: FastMap::default(),
            driver_devices: FastMap::default(),
        }
    }

    /// Register a new USB device.
    ///
    /// Devices will automatically be registered by the XHCI controller.
    pub fn register_device(&mut self, device_id: u32, device: UsbDevice) -> Result<(), UsbError> {
        for (driver_name, driver) in &self.drivers {
            if driver.should_bind(&device) {
                log::info!(
                    "Binding USB device on port {} to driver {}",
                    device.port_id(),
                    driver_name
                );

                driver.init(&device)?;
                self.driver_devices
                    .entry(driver_name)
                    .or_default()
                    .push(device_id);
            }
        }

        self.devices.insert(device_id, device);

        Ok(())
    }

    /// Add a driver to the hub manually.
    pub fn add_driver(&mut self, driver: impl UsbDriver) {
        self.drivers.insert(driver.name(), Box::new(driver));
    }

    /// Get all registered driver names.
    pub fn get_drivers(&self) -> Vec<&'static str> {
        self.drivers.keys().copied().collect()
    }

    /// Get a driver and cast it to a specific type safely/unsafely.
    ///
    /// # Safety
    ///
    /// The caller must ensure the given name actually matches the [UsbDriver::name()].
    pub unsafe fn get_driver_cast<D: UsbDriver>(&self, name: &'static str) -> Option<&D> {
        let boxed = self.get_driver(name).ok()?;
        let trait_ref: &dyn UsbDriver = &**boxed;
        let raw_ptr = trait_ref as *const dyn UsbDriver as *const D;
        Some(unsafe { &*raw_ptr })
    }

    /// Get all devices operated on by the given driver.
    pub fn devices_of_driver(&self, driver_name: &'static str) -> Option<Vec<&UsbDevice>> {
        self.driver_devices.get(driver_name).map(|device_ids| {
            device_ids
                .iter()
                .filter_map(|id| self.devices.get(id))
                .collect()
        })
    }

    /// Forward raw interrupt reports to bound drivers.
    pub fn dispatch_report(&self, device_id: u32, report: &[u8]) {
        if let Some(device) = self.devices.get(&device_id) {
            for (driver_name, device_ids) in &self.driver_devices {
                if device_ids.contains(&device_id)
                    && let Some(driver) = self.drivers.get(driver_name)
                {
                    driver.handle_report(device, report);
                }
            }
        }
    }
}

impl DeviceHub for UsbDeviceHub {
    type Device = UsbDevice;
    type DeviceId = u32;
    type Driver = Box<dyn UsbDriver>;
    type DriverId = &'static str;
    type Error = UsbError;

    fn init(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn register(&mut self, driver: Self::Driver) -> Result<(), Self::Error> {
        if self.drivers.contains_key(driver.name()) {
            return Err(UsbError::InvalidDevice);
        }

        let mut bound_device_ids = Vec::new();
        let device_ids: Vec<u32> = self.devices.keys().copied().collect();

        for id in device_ids {
            let device = self.devices.get(&id).ok_or(UsbError::InvalidDevice)?;

            if driver.should_bind(device) {
                log::info!(
                    "Binding USB device on port {} to driver {}",
                    device.port_id(),
                    driver.name()
                );
                driver.init(device)?;
                bound_device_ids.push(id);
            }
        }

        self.driver_devices.insert(driver.name(), bound_device_ids);
        self.drivers.insert(driver.name(), driver);

        Ok(())
    }

    fn unregister(&mut self, driver_name: &'static str) -> Result<Self::Driver, Self::Error> {
        let driver = self
            .drivers
            .remove(driver_name)
            .ok_or(UsbError::InvalidDevice)?;

        let devices = self
            .driver_devices
            .remove(driver.name())
            .unwrap_or_default();

        for id in devices {
            if let Ok(device) = self.get(id) {
                let _ = driver.destroy(device);
            }
        }

        Ok(driver)
    }

    fn devices(&self) -> Vec<Self::DeviceId> {
        self.devices.keys().copied().collect()
    }

    fn get(&self, id: Self::DeviceId) -> Result<&Self::Device, Self::Error> {
        self.devices.get(&id).ok_or(UsbError::InvalidDevice)
    }

    fn get_driver(&self, driver: Self::DriverId) -> Result<&Self::Driver, Self::Error> {
        self.drivers.get(driver).ok_or(UsbError::InvalidDevice)
    }
}

/// Standard USB device descriptor layout
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct UsbDeviceDescriptor {
    /// Descriptor length.
    pub length: u8,
    /// Descriptor type.
    pub descriptor_type: u8,
    /// BCD USB
    pub bcd_usb: u16,
    /// Device class.
    pub device_class: u8,
    /// Device subclass.
    pub device_subclass: u8,
    /// Device protocol.
    pub device_protocol: u8,
    /// Maximum packet size.
    pub max_packet_size0: u8,
    /// Vendor ID.
    pub vendor_id: u16,
    /// Product ID.
    pub product_id: u16,
    /// BCD Device.
    pub bcd_device: u16,
    /// Manufacturer index.
    pub manufacturer_idx: u8,
    /// Product index.
    pub product_idx: u8,
    /// Serial number index.
    pub serial_number_idx: u8,
    /// Number of configurations.
    pub num_configurations: u8,
}

/// A handle to an active USB device managed by the xHCI controller backend.
pub trait UsbDeviceHandle: Send + Sync {
    /// Perform a synchronous USB control transfer (Endpoint 0).
    fn control_transfer(
        &self,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        data: &mut [u8],
    ) -> Result<usize, UsbError>;

    /// Submit an interrupt transfer request for an interrupt endpoint.
    fn submit_interrupt_transfer(&self, endpoint: u8, buffer: &mut [u8])
    -> Result<usize, UsbError>;
}

/// A USB device instance connected to a port.
pub struct UsbDevice {
    port_id: u8,
    descriptor: UsbDeviceDescriptor,
    handle: Box<dyn UsbDeviceHandle>,
}

impl UsbDevice {
    /// Create a new [UsbDevice].
    pub fn new(
        port_id: u8,
        descriptor: UsbDeviceDescriptor,
        handle: Box<dyn UsbDeviceHandle>,
    ) -> Self {
        Self {
            port_id,
            descriptor,
            handle,
        }
    }

    /// Returns the USB port ID.
    pub fn port_id(&self) -> u8 {
        self.port_id
    }

    /// Returns the USB device descriptor.
    pub fn descriptor(&self) -> &UsbDeviceDescriptor {
        &self.descriptor
    }

    /// Returns the underlying device communication handle.
    pub fn handle(&self) -> &dyn UsbDeviceHandle {
        &*self.handle
    }
}

impl crate::device::Device for UsbDevice {
    type DeviceId = u32;
    type Error = UsbError;
}

/// A trait defining protocol-specific USB drivers (e.g., USB Keyboards, Mice).
pub trait UsbDriver: Send + Sync + 'static {
    /// Unique name of the USB driver.
    fn name(&self) -> &'static str;

    /// Inspects the device to check if this driver should bind.
    fn should_bind(&self, device: &UsbDevice) -> bool;

    /// Initialize the USB device driver.
    fn init(&self, device: &UsbDevice) -> Result<(), UsbError>;

    /// Handle incoming interrupt transfer reports (e.g., raw 8-byte keyboard report).
    fn handle_report(&self, device: &UsbDevice, report: &[u8]);

    /// Clean up and destroy the device driver association.
    fn destroy(&self, device: &UsbDevice) -> Result<(), UsbError> {
        let _ = device;
        Ok(())
    }
}
