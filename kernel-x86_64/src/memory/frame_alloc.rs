use crate::memory::phys_mem_offset;
use core::ptr;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use kernel_core::requests;
use limine::memmap::MEMMAP_USABLE;
use x86_64::PhysAddr;
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};

/// Global atomic, lockless frame allocator
static mut FRAME_ALLOCATOR: AtomicFrameAllocator = AtomicFrameAllocator::new();

/// Returns a mutable reference to the global frame allocator.
///
/// # Safety
///
/// Callers must not actually mutate the returned reference.
/// It just returns a mutable reference, because the page mapper requires one.
pub fn get<'a>() -> &'a mut AtomicFrameAllocator {
    unsafe { &mut FRAME_ALLOCATOR }
}

pub struct AtomicFrameAllocator {
    head: AtomicU64,
    free_count: AtomicUsize,
}

impl AtomicFrameAllocator {
    pub const fn new() -> Self {
        Self {
            head: AtomicU64::new(0),
            free_count: AtomicUsize::new(0),
        }
    }

    /// Initialize the allocator from usable memory map regions.
    ///
    /// # Safety
    ///
    /// Must be called once during system boot before concurrent allocations begin.
    pub unsafe fn init(&self) {
        let phys_mem_offset = phys_mem_offset();

        for region in requests::memory_map().entries() {
            if region.type_ != MEMMAP_USABLE {
                continue;
            }

            let start = region.base;
            let end = region.base + region.length;

            let mut addr = start;
            while addr < end {
                let frame = PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(addr));
                unsafe {
                    self.deallocate_frame_internal(frame, phys_mem_offset);
                }
                addr += 4096;
            }
        }
    }

    /// Free/Push a physical frame back onto the lockless stack.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the frame is not currently in use.
    pub unsafe fn deallocate_frame(&self, frame: PhysFrame<Size4KiB>) {
        let phys_mem_offset = phys_mem_offset();
        unsafe {
            self.deallocate_frame_internal(frame, phys_mem_offset);
        }
    }

    /// Allocate/Pop a physical frame from the lockless stack.
    pub fn allocate_frame_internal(&self) -> Option<PhysFrame<Size4KiB>> {
        let phys_mem_offset = phys_mem_offset();
        let mut current_head = self.head.load(Ordering::Acquire);

        loop {
            if current_head == 0 {
                return None; // Out of memory
            }

            // Read the next frame pointer stored inside the head frame
            let virt_ptr = (current_head + phys_mem_offset) as *const u64;
            let next_phys_addr = unsafe { ptr::read_volatile(virt_ptr) };

            // Attempt to update head pointer to next_phys_addr
            match self.head.compare_exchange_weak(
                current_head,
                next_phys_addr,
                Ordering::Release,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    self.free_count.fetch_sub(1, Ordering::Relaxed);
                    return Some(PhysFrame::containing_address(PhysAddr::new(current_head)));
                }
                Err(actual) => current_head = actual,
            }
        }
    }

    /// Get the remaining number of free 4KiB frames.
    pub fn free_count(&self) -> usize {
        self.free_count.load(Ordering::Relaxed)
    }

    unsafe fn deallocate_frame_internal(&self, frame: PhysFrame<Size4KiB>, phys_mem_offset: u64) {
        let phys_addr = frame.start_address().as_u64();
        let virt_ptr = (phys_addr + phys_mem_offset) as *mut u64;

        let mut current_head = self.head.load(Ordering::Relaxed);

        loop {
            // Write current head physical address into the first 8 bytes of the returning frame
            unsafe { ptr::write_volatile(virt_ptr, current_head) };

            // Attempt to point head to new frame
            match self.head.compare_exchange_weak(
                current_head,
                phys_addr,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    self.free_count.fetch_add(1, Ordering::Relaxed);
                    break;
                }
                Err(actual) => current_head = actual,
            }
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for AtomicFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        self.allocate_frame_internal()
    }
}

unsafe impl FrameAllocator<Size4KiB> for &AtomicFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        self.allocate_frame_internal()
    }
}
