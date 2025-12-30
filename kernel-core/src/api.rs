use crate::info::KernelApiInfo;
use crate::sync::init::InitData;
use crate::time::Time;
use core::alloc::Layout;

/// The size of the kernel heap in bytes.
///
/// The [KernelApi] should use this to allocate enough memory for the kernel.
///
/// As of right now, it's equal to 16 MB.
pub const HEAP_SIZE: usize = 16 * 1024 * 1024;

static API: InitData<KernelApi> = InitData::uninit();

/// Get the global [KernelApi].
pub const fn kernel() -> KernelApi {
    *API.get()
}

/// Sets the global [KernelApi].
///
/// Should only be called once right after booting.
pub const unsafe fn set(kernel: KernelApi) -> KernelApi {
    unsafe {
        let api = *API.init(kernel);
        api
    }
}

/// Get the global [KernelApiInfo].
pub const fn info() -> KernelApiInfo {
    kernel().info
}

/// Executes the halt instruction to halt the CPU.
pub fn halt() {
    (kernel().halt)();
}

/// Get the global [PortApi].
pub const fn port() -> PortApi {
    kernel().port
}

/// Get the global [MemoryApi].
pub const fn memory() -> MemoryApi {
    kernel().memory
}

/// Get the global [TimeApi].
pub const fn time() -> TimeApi {
    kernel().time
}

/// Disable interrupts on the system.
pub fn disable_interrupts() {
    (kernel().disable_interrupts)();
}

/// Enable interrupts on the system.
pub fn enable_interrupts() {
    (kernel().enable_interrupts)();
}

/// Executes the given function without interrupts.
pub fn without_interrupts<F: FnOnce()>(f: F) {
    disable_interrupts();
    f();
    enable_interrupts();
}

/// Kernel API to abstract over common kernel functionalities.
///
/// Used to implement kernel functions independent of architecture.
///
/// This should be the entry point for new architecture-specific kernel implementations.
#[derive(Copy, Clone)]
pub struct KernelApi {
    /// The [KernelApiInfo] for information about the package.
    pub info: KernelApiInfo,
    /// The init function. Should initialize everything up to the heap allocator.
    ///
    /// For general setup (where the heap is already available), see `setup`.
    pub init: unsafe fn(),
    /// The setup function. Run after `init`. Should initialize remaining systems.
    pub setup: unsafe fn(),
    /// The halt function to move the CPU into an idle state.
    pub halt: fn(),
    /// Disable interrupts on the system.
    pub disable_interrupts: fn(),
    /// Enable interrupts on the system.
    pub enable_interrupts: fn(),
    /// The [PortApi] for port communication.
    pub port: PortApi,
    /// The [MemoryApi] for memory management.
    pub memory: MemoryApi,
    /// The [TimeApi] for time reading.
    pub time: TimeApi,
}

/// Port API of the kernel.
///
/// Used to communicate with ports.
#[derive(Copy, Clone)]
pub struct PortApi {
    /// Read a `u8` value from a port.
    pub read_u8: unsafe fn(port: u16) -> u8,
    /// Write a `u8` value to a port.
    pub write_u8: unsafe fn(port: u16, value: u8),
    /// Read a `u16` value from a port.
    pub read_u16: unsafe fn(port: u16) -> u16,
    /// Write a `u16` value to a port.
    pub write_u16: unsafe fn(port: u16, value: u16),
    /// Read a `u32` value from a port.
    pub read_u32: unsafe fn(port: u16) -> u32,
    /// Write a `u32` value to a port.
    pub write_u32: unsafe fn(port: u16, value: u32),
}

impl PortApi {
    /// Read a `u8` value from a port.
    pub unsafe fn read_u8(&self, port: u16) -> u8 {
        unsafe { (self.read_u8)(port) }
    }

    /// Write a `u8` value to a port.
    pub unsafe fn write_u8(&self, port: u16, value: u8) {
        unsafe { (self.write_u8)(port, value) }
    }

    /// Read a `u16` value from a port.
    pub unsafe fn read_u16(&self, port: u16) -> u16 {
        unsafe { (self.read_u16)(port) }
    }

    /// Write a `u16` value to a port.
    pub unsafe fn write_u16(&self, port: u16, value: u16) {
        unsafe { (self.write_u16)(port, value) }
    }

    /// Read a `u32` value from a port.
    pub unsafe fn read_u32(&self, port: u16) -> u32 {
        unsafe { (self.read_u32)(port) }
    }

    /// Write a `u32` value to a port.
    pub unsafe fn write_u32(&self, port: u16, value: u32) {
        unsafe { (self.write_u32)(port, value) }
    }
}

/// The memory-management API of the kernel.
#[derive(Copy, Clone, Debug)]
pub struct MemoryApi {
    /// Return if the allocator is initialized.
    ///
    /// This should return `true`, once the kernel setup is completed.
    pub is_init: fn() -> bool,
    /// See [core::alloc::GlobalAlloc::alloc].
    pub alloc: unsafe fn(layout: Layout) -> *mut u8,
    /// See [core::alloc::GlobalAlloc::alloc_zeroed].
    pub alloc_zeroed: unsafe fn(layout: Layout) -> *mut u8,
    /// See [core::alloc::GlobalAlloc::dealloc].
    pub dealloc: unsafe fn(ptr: *mut u8, layout: Layout),
    /// See [core::alloc::GlobalAlloc::realloc].
    pub realloc: unsafe fn(ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8,
}

impl MemoryApi {
    /// Return if the allocator is initialized.
    ///
    /// This should return `true`, once the kernel setup is completed.
    pub fn is_init(&self) -> bool {
        (self.is_init)()
    }

    /// See [core::alloc::GlobalAlloc::alloc].
    pub unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { (self.alloc)(layout) }
    }

    /// See [core::alloc::GlobalAlloc::realloc].
    pub unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        unsafe { (self.alloc_zeroed)(layout) }
    }

    /// See [core::alloc::GlobalAlloc::dealloc].
    pub unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { (self.dealloc)(ptr, layout) }
    }

    /// See [core::alloc::GlobalAlloc::realloc].
    pub unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        unsafe { (self.realloc)(ptr, layout, new_size) }
    }
}

/// The time API for the kernel.
///
/// Responsible for getting the current system time.
///
/// *NOTE:* This is not responsible for timed intervals or scheduling stuff.
#[derive(Copy, Clone)]
pub struct TimeApi {
    /// Get the current system time.
    pub read: fn() -> Time,
}

impl TimeApi {
    /// Read the time from the internal clock.
    pub fn read(&self) -> Time {
        (self.read)()
    }
}
