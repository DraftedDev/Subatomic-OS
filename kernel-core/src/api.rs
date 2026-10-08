use crate::info::KernelApiInfo;
use crate::sync::init::InitData;
use core::alloc::Layout;
use time::{OffsetDateTime, UtcDateTime};

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
/// # Safety
///
/// This must only be called exactly once before any API usage.
pub const unsafe fn set(kernel: KernelApi) -> KernelApi {
    unsafe { *API.init(kernel) }
}

/// Get the global [KernelApiInfo].
pub const fn info() -> KernelApiInfo {
    kernel().info
}

/// Get the global [InterruptsApi].
pub const fn interrupts() -> InterruptsApi {
    kernel().interrupts
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

/// Executes the given function without interrupts.
pub fn without_interrupts<R, F: FnOnce() -> R>(f: F) -> R {
    (interrupts().disable_interrupts)();
    let result = f();
    (interrupts().enable_interrupts)();
    result
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
    /// Generate a seed for the random number generator.
    pub seed: fn(quality: bool) -> u64,
    /// The [InterruptsApi] for interrupt control.
    pub interrupts: InterruptsApi,
    /// The [PortApi] for port communication.
    pub port: PortApi,
    /// The [MemoryApi] for memory management.
    pub memory: MemoryApi,
    /// The [TimeApi] for time reading.
    pub time: TimeApi,
}

impl KernelApi {
    /// Halts the CPU into an idle state.
    pub fn halt(&self) {
        (self.halt)()
    }

    /// Generate a seed for the random number generator.
    pub fn seed(&self, quality: bool) -> u64 {
        (self.seed)(quality)
    }
}

/// The interrupt API for the kernel.
///
/// Used to control interrupts.
#[derive(Copy, Clone)]
pub struct InterruptsApi {
    /// Disable interrupts on the system.
    pub disable_interrupts: fn(),
    /// Enable interrupts on the system.
    pub enable_interrupts: fn(),
    /// Signals the end of the current interrupt.
    pub end_of_interrupt: unsafe fn(),
    /// Get the ID of the current interrupt controller.
    pub interrupt_ctrl_id: fn() -> u32,
}

impl InterruptsApi {
    /// Disable interrupts on the system.
    pub fn disable_interrupts(&self) {
        (self.disable_interrupts)()
    }

    /// Enable interrupts on the system.
    pub fn enable_interrupts(&self) {
        (self.enable_interrupts)()
    }

    /// Signals the end of the current interrupt.
    ///
    /// # Safety
    ///
    /// The caller must ensure, this function is called inside the correct interrupt context.
    pub unsafe fn end_of_interrupt(&self) {
        (self.enable_interrupts)()
    }

    /// Get the ID of the current interrupt controller.
    pub fn interrupt_ctrl_id(&self) -> u32 {
        (self.interrupt_ctrl_id)()
    }
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
    ///
    /// # Safety
    ///
    /// Port I/O is generally unsafe, since unintended side effects may occur.
    pub unsafe fn read_u8(&self, port: u16) -> u8 {
        unsafe { (self.read_u8)(port) }
    }

    /// Write a `u8` value to a port.
    ///
    /// # Safety
    ///
    /// Port I/O is generally unsafe, since unintended side effects may occur.
    pub unsafe fn write_u8(&self, port: u16, value: u8) {
        unsafe { (self.write_u8)(port, value) }
    }

    /// Read a `u16` value from a port.
    ///
    /// # Safety
    ///
    /// Port I/O is generally unsafe, since unintended side effects may occur.
    pub unsafe fn read_u16(&self, port: u16) -> u16 {
        unsafe { (self.read_u16)(port) }
    }

    /// Write a `u16` value to a port.
    ///
    /// # Safety
    ///
    /// Port I/O is generally unsafe, since unintended side effects may occur.
    pub unsafe fn write_u16(&self, port: u16, value: u16) {
        unsafe { (self.write_u16)(port, value) }
    }

    /// Read a `u32` value from a port.
    ///
    /// # Safety
    ///
    /// Port I/O is generally unsafe, since unintended side effects may occur.
    pub unsafe fn read_u32(&self, port: u16) -> u32 {
        unsafe { (self.read_u32)(port) }
    }

    /// Write a `u32` value to a port.
    ///
    /// # Safety
    ///
    /// Port I/O is generally unsafe, since unintended side effects may occur.
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
    /// Translates a physical address to a virtual address.
    pub translate: unsafe fn(addr: usize) -> usize,
    /// Maps the given physical address to a virtual address.
    pub map_to: unsafe fn(addr: usize, writable: bool, cache: bool) -> usize,
    /// Allocates contiguous physical DMA memory.
    pub dma_alloc: unsafe fn(pages: usize) -> Option<(usize, usize)>,
    /// Deallocates contiguous physical DMA memory.
    pub dma_dealloc: unsafe fn(phys_addr: usize, virt_addr: usize, pages: usize),
    /// Prepares a buffer for DMA.
    pub dma_sync_device: unsafe fn(addr: usize, size: usize, dir: DmaDirection),
    /// Cleans up after DMA.
    pub dma_sync_cpu: unsafe fn(addr: usize, size: usize, dir: DmaDirection),
}

impl MemoryApi {
    /// Return if the allocator is initialized.
    ///
    /// This should return `true`, once the kernel setup is completed.
    pub fn is_init(&self) -> bool {
        (self.is_init)()
    }

    /// See [core::alloc::GlobalAlloc::alloc].
    ///
    /// # Safety
    ///
    /// The specified layout must be correct.
    pub unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { (self.alloc)(layout) }
    }

    /// See [core::alloc::GlobalAlloc::realloc].
    ///
    /// # Safety
    ///
    /// The specified layout must be correct.
    pub unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        unsafe { (self.alloc_zeroed)(layout) }
    }

    /// See [core::alloc::GlobalAlloc::dealloc].
    ///
    /// # Safety
    ///
    /// The specified layout and pointer must be correct.
    pub unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { (self.dealloc)(ptr, layout) }
    }

    /// See [core::alloc::GlobalAlloc::realloc].
    ///
    /// # Safety
    ///
    /// The specified pointer, layout and new size must be correct.
    pub unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        unsafe { (self.realloc)(ptr, layout, new_size) }
    }

    /// Translates a physical address to a virtual address.
    ///
    /// # Safety
    ///
    /// The specified address must be valid.
    pub unsafe fn translate(&self, addr: usize) -> usize {
        unsafe { (self.translate)(addr) }
    }

    /// Maps the given physical address to a virtual address.
    ///
    /// # Safety
    ///
    /// The specified address must be valid.
    pub unsafe fn map_to(&self, addr: usize, writable: bool, cache: bool) -> usize {
        unsafe { (self.map_to)(addr, writable, cache) }
    }

    /// Allocates contiguous physical DMA memory.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the requested number of pages is available
    /// and that the returned addresses are used correctly.
    pub unsafe fn dma_alloc(&self, pages: usize) -> Option<(usize, usize)> {
        unsafe { (self.dma_alloc)(pages) }
    }

    /// Deallocates contiguous physical DMA memory.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the specified physical and virtual addresses are valid
    /// and correspond to a previously allocated DMA region, and that the number of pages is correct.
    pub unsafe fn dma_dealloc(&self, phys_addr: usize, virt_addr: usize, pages: usize) {
        unsafe { (self.dma_dealloc)(phys_addr, virt_addr, pages) }
    }

    /// Prepares a buffer for DMA.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the specified address and size are valid
    /// and correspond to a buffer that will be used for DMA, and that the direction is correct.
    pub unsafe fn dma_sync_device(&self, addr: usize, size: usize, dir: DmaDirection) {
        unsafe { (self.dma_sync_device)(addr, size, dir) }
    }

    /// Cleans up after DMA.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the specified address and size are valid
    /// and correspond to a buffer that was used for DMA, and that the direction is correct.
    pub unsafe fn dma_sync_cpu(&self, addr: usize, size: usize, dir: DmaDirection) {
        unsafe { (self.dma_sync_cpu)(addr, size, dir) }
    }
}

/// The direction of a DMA transfer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmaDirection {
    /// Transfer to device.
    ToDevice,
    /// Transfer from device.
    FromDevice,
    /// Transfer in both directions.
    Bidirectional,
}

/// The time API for the kernel.
///
/// Responsible for getting the current system time.
///
/// *NOTE:* This is not responsible for timed intervals or scheduling stuff.
#[derive(Copy, Clone)]
pub struct TimeApi {
    /// Get the local system time.
    pub read_local: fn() -> OffsetDateTime,
    /// Get the UTC system time.
    pub read_utc: fn() -> UtcDateTime,
    /// Set the offset of the local time.
    pub set_offset: fn(hours: i8, minutes: i8, seconds: i8),
}

impl TimeApi {
    /// Read the local system time from the internal clock.
    pub fn read_local(&self) -> OffsetDateTime {
        (self.read_local)()
    }

    /// Read the UTC system time from the internal clock.
    pub fn read_utc(&self) -> UtcDateTime {
        (self.read_utc)()
    }

    /// Set the offset of the local time.
    pub fn set_offset(&self, hours: i8, minutes: i8, seconds: i8) {
        (self.set_offset)(hours, minutes, seconds)
    }
}
