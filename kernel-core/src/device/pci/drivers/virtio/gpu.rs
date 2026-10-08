use crate::{
    api,
    device::pci::{
        InterruptStatus, PciDevice, PciDriver, drivers::virtio::VirtIoPciAccess, error::PciError,
        hal::KernelHal,
    },
    sync::mutex::Mutex,
};
use virtio_drivers::{
    device::gpu::VirtIOGpu,
    transport::pci::{
        PciTransport,
        bus::{DeviceFunction, PciRoot},
    },
};

/// VirtIO GPU Driver.
pub struct VirtioGpuDriver {
    device: Mutex<Option<VirtIOGpu<KernelHal, PciTransport>>>,
}

impl VirtioGpuDriver {
    /// Create a new driver instance.
    pub fn new() -> Self {
        Self {
            device: Mutex::new(None),
        }
    }

    fn with_device<R>(
        &self,
        func: impl FnOnce(&mut VirtIOGpu<KernelHal, PciTransport>) -> R,
    ) -> Option<R> {
        self.device.run(|dev| dev.as_mut().map(func))
    }
}

impl PciDriver for VirtioGpuDriver {
    fn name(&self) -> &'static str {
        "virtio-gpu-driver"
    }

    fn should_bind(&self, device: &PciDevice) -> bool {
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

        let gpu = VirtIOGpu::new(transport).map_err(anyhow::Error::from)?;

        self.device.run(|dev| *dev = Some(gpu));

        Ok(())
    }

    fn handle_interrupt(&self, _: &PciDevice) -> InterruptStatus {
        self.with_device(|gpu| {
            gpu.ack_interrupt();
            InterruptStatus::Handled
        })
        .unwrap_or(InterruptStatus::Ignored)
    }

    fn destroy(&self, _: &PciDevice) -> Result<(), PciError> {
        self.device.run(|dev| *dev = None);

        Ok(())
    }
}
