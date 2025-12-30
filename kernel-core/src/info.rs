use crate::api::kernel;

/// Information about this kernel.
#[derive(Copy, Clone, Debug)]
pub struct KernelInfo {
    /// Information about the core kernel library.
    pub core: KernelCoreInfo,
    /// Information about the [KernelApi](crate::api::KernelApi).
    pub api: KernelApiInfo,
}

impl KernelInfo {
    /// Fetch the kernel information.
    pub const fn fetch() -> Self {
        Self {
            core: KernelCoreInfo {
                package: env!("CARGO_PKG_NAME"),
                version: env!("CARGO_PKG_VERSION"),
            },
            api: kernel().info,
        }
    }
}

/// Information about the core kernel library.
#[derive(Copy, Clone, Debug)]
pub struct KernelCoreInfo {
    /// The package name.
    pub package: &'static str,
    /// The version.
    pub version: &'static str,
}

/// Information about the [KernelApi](crate::api::KernelApi).
#[derive(Copy, Clone, Debug)]
pub struct KernelApiInfo {
    /// The package name.
    pub package: &'static str,
    /// The version.
    pub version: &'static str,
}
