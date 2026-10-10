use crate::{
    api,
    control::display::{Display, PixelFormat},
    device::pci::{
        InterruptStatus, PCI_HUB, PciDevice, PciDriver, drivers::virtio::VirtIoPciAccess,
        error::PciError, hal::KernelHal,
    },
    sync::mutex::Mutex,
};
use alloc::boxed::Box;
use virtio_drivers::{
    device::gpu::VirtIOGpu,
    transport::pci::{
        PciTransport,
        bus::{DeviceFunction, PciRoot},
    },
};

const NAME: &str = "virtio-gpu-driver";

/// VirtIO GPU Driver.
pub struct VirtioGpuDriver {
    device: Mutex<Option<DriverState>>,
}

impl VirtioGpuDriver {
    /// Fetch the driver from the global PCI device hub and run the closure.
    ///
    /// Will panic if the driver is not registered and initialized yet.
    pub fn fetch<R>(f: impl FnOnce(&mut DriverState) -> R) -> R {
        PCI_HUB.get().run(|hub| {
            let driver = unsafe { hub.get_driver_cast::<VirtioGpuDriver>(NAME) }
                .expect("Driver not initialized");

            driver.with_state(f).expect("Driver state not initialized")
        })
    }

    /// Create a new driver instance.
    pub fn new() -> Self {
        Self {
            device: Mutex::new(None),
        }
    }

    fn with_state<R>(&self, func: impl FnOnce(&mut DriverState) -> R) -> Option<R> {
        self.device.run(|dev| dev.as_mut().map(func))
    }
}

impl PciDriver for VirtioGpuDriver {
    fn name(&self) -> &'static str {
        NAME
    }

    fn should_bind(&self, device: &PciDevice) -> bool {
        // Driver already bound
        if self.device.run(|dev| dev.is_some()) {
            return false;
        }

        let (vendor_id, device_id) = device.id();

        vendor_id == 0x1AF4 && device_id == 0x1050
    }

    fn should_attach_msi(&self, _: &PciDevice) -> Option<u8> {
        Some(api::interrupts().interrupt_ctrl_id() as u8)
    }

    fn init(&self, device: &PciDevice, _: Option<u8>) -> Result<(), PciError> {
        device.enable_bus_mastering();

        let access = VirtIoPciAccess::new(device.config(), device.addr());
        let transport = PciTransport::new::<KernelHal, VirtIoPciAccess>(
            &mut PciRoot::new(access),
            DeviceFunction {
                bus: device.addr().bus(),
                device: device.addr().device(),
                function: device.addr().function(),
            },
        )
        .map_err(anyhow::Error::from)?;

        let mut gpu = VirtIOGpu::new(transport).map_err(anyhow::Error::from)?;

        let res = gpu.resolution().map_err(anyhow::Error::from)?;
        let framebuffer = gpu.setup_framebuffer().map_err(anyhow::Error::from)?;

        let framebuffer = unsafe {
            let ptr = framebuffer.as_ptr() as *mut u8;
            let len = framebuffer.len();
            core::slice::from_raw_parts_mut(ptr, len)
        };

        self.device.run(|dev| *dev = Some(DriverState { gpu }));

        let display = GpuDisplay {
            width: res.0,
            height: res.1,
            framebuffer: &mut *framebuffer,
        };

        log::info!("Initializing GPU display...");
        crate::control::display::DISPLAY.init(Box::new(display));

        Ok(())
    }

    fn handle_interrupt(&self, _: &PciDevice) -> InterruptStatus {
        self.with_state(|state| {
            state.gpu.ack_interrupt();
            InterruptStatus::Handled
        })
        .unwrap_or(InterruptStatus::Ignored)
    }

    fn destroy(&self, _: &PciDevice) -> Result<(), PciError> {
        self.device.run(|dev| *dev = None);

        Ok(())
    }
}

/// The internal VirtIO GPU driver state.
pub struct DriverState {
    /// The internal gpu driver device.
    pub gpu: VirtIOGpu<KernelHal, PciTransport>,
}

/// A gpu-backed display.
pub struct GpuDisplay {
    width: u32,
    height: u32,
    framebuffer: &'static [u8],
}

impl Display for GpuDisplay {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn stride(&self) -> u32 {
        self.width * 4
    }

    fn format(&self) -> PixelFormat {
        PixelFormat::Bgra8888
    }

    fn buffer(&mut self) -> &mut [u8] {
        unsafe {
            let ptr = self.framebuffer.as_ptr() as *mut u8;
            let len = self.framebuffer.len();
            core::slice::from_raw_parts_mut(ptr, len)
        }
    }

    fn flush(&mut self) {
        VirtioGpuDriver::fetch(|state| {
            state
                .gpu
                .flush()
                .unwrap_or_else(|err| log::error!("Failed to flush virtio GPU buffer: {err}"));
        });
    }
}
