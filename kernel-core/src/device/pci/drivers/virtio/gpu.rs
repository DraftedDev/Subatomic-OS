use crate::{
    api,
    control::display::{Display, PixelFormat},
    device::pci::{
        InterruptStatus, PCI_HUB, PciDevice, PciDriver, drivers::virtio::VirtIoPciAccess,
        error::PciError, hal::KernelHal,
    },
    sync::mutex::Mutex,
    wrapper::SendSyncWrapper,
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
        unsafe { PCI_HUB.get_unsafe() }.run(|hub| {
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

        let (width, height) = gpu
            .edid_preferred_resolution()
            .map_err(anyhow::Error::from)?;
        let framebuffer_slice = gpu
            .change_resolution(width, height)
            .map_err(anyhow::Error::from)?;

        let ptr = framebuffer_slice.as_ptr() as *mut u8;
        let len = framebuffer_slice.len();
        let stride = width * 4;

        let display = unsafe { GpuDisplay::new(width, height, stride, ptr, len) };

        log::info!(
            "Initializing GPU display with dimensions {}x{}...",
            width,
            height
        );

        self.device.run(|dev| *dev = Some(DriverState { gpu }));

        crate::control::display::DISPLAY.init(Box::new(display));

        crate::control::CONTROL
            .get()
            .run(|ctrl| ctrl.refresh_display());

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
    stride: u32,
    framebuffer_ptr: SendSyncWrapper<*mut u8>,
    framebuffer_len: usize,
}

impl GpuDisplay {
    /// Create a new [GpuDisplay].
    ///
    /// # Safety
    ///
    /// The given pointer and slice length must be correct.
    pub unsafe fn new(width: u32, height: u32, stride: u32, ptr: *mut u8, len: usize) -> Self {
        Self {
            width,
            height,
            stride,
            framebuffer_ptr: unsafe { SendSyncWrapper::new(ptr) },
            framebuffer_len: len,
        }
    }
}

impl Display for GpuDisplay {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn stride(&self) -> u32 {
        self.stride
    }

    fn format(&self) -> PixelFormat {
        PixelFormat::Bgra8888
    }

    fn buffer(&mut self) -> &mut [u8] {
        unsafe {
            let ptr = *self.framebuffer_ptr.inner();
            assert!(!ptr.is_null(), "Framebuffer pointer is null");
            core::slice::from_raw_parts_mut(ptr, self.framebuffer_len)
        }
    }

    fn flush(&mut self) {
        VirtioGpuDriver::fetch(|state| {
            state.gpu.flush().unwrap_or_else(|err| {
                log::error!("Failed to flush virtio GPU buffer: {err}");
            });
        });
    }
}
