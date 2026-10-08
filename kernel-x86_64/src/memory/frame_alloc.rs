use crate::memory::{PHYS_MEM_OFFSET, phys_mem_offset};
use core::ptr;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use kernel_core::requests;
use limine::memmap::MEMMAP_USABLE;
use x86_64::PhysAddr;
use x86_64::structures::paging::{FrameAllocator, FrameDeallocator, PhysFrame, Size4KiB};

const MAX_REGIONS: usize = 32;

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
    regions: [PhysRegion; MAX_REGIONS],
    region_count: usize,
    current_region: AtomicUsize,
    current_addr: AtomicU64,
    free_count: AtomicUsize,
}

impl AtomicFrameAllocator {
    pub const fn new() -> Self {
        Self {
            head: AtomicU64::new(0),
            regions: [PhysRegion { start: 0, end: 0 }; MAX_REGIONS],
            region_count: 0,
            current_region: AtomicUsize::new(0),
            current_addr: AtomicU64::new(0),
            free_count: AtomicUsize::new(0),
        }
    }

    /// Initialize the frame allocator with the memory map provided by the bootloader.
    ///
    /// # Safety
    ///
    /// This function must only be called once to initialize the allocator.
    pub unsafe fn init(&mut self) {
        let mut total_frames = 0;
        let mut reg_idx = 0;

        for region in requests::memory_map().entries() {
            if region.type_ != MEMMAP_USABLE || reg_idx >= MAX_REGIONS {
                continue;
            }

            let start = region.base;
            let end = region.base + region.length;
            let frame_count = (end - start) / 4096;

            if frame_count > 0 {
                self.regions[reg_idx] = PhysRegion { start, end };
                total_frames += frame_count as usize;
                reg_idx += 1;
            }
        }

        self.region_count = reg_idx;
        self.free_count.store(total_frames, Ordering::Relaxed);

        if reg_idx > 0 {
            self.current_addr
                .store(self.regions[0].start, Ordering::Relaxed);
        }
    }

    fn allocate_frame_internal(&self) -> Option<PhysFrame<Size4KiB>> {
        let phys_mem_offset = phys_mem_offset();

        let mut current_head = self.head.load(Ordering::Acquire);
        while current_head != 0 {
            let virt_ptr = (current_head + phys_mem_offset) as *const u64;
            let next_phys_addr = unsafe { ptr::read_volatile(virt_ptr) };

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

        let mut reg_idx = self.current_region.load(Ordering::Acquire);

        while reg_idx < self.region_count {
            let region = &self.regions[reg_idx];
            let addr = self.current_addr.load(Ordering::Acquire);

            if addr < region.end {
                match self.current_addr.compare_exchange_weak(
                    addr,
                    addr + 4096,
                    Ordering::Release,
                    Ordering::Acquire,
                ) {
                    Ok(_) => {
                        self.free_count.fetch_sub(1, Ordering::Relaxed);
                        return Some(PhysFrame::containing_address(PhysAddr::new(addr)));
                    }
                    Err(_) => continue,
                }
            } else {
                // Move to next region
                let _ = self.current_region.compare_exchange(
                    reg_idx,
                    reg_idx + 1,
                    Ordering::Release,
                    Ordering::Acquire,
                );
                reg_idx = self.current_region.load(Ordering::Acquire);
                if reg_idx < self.region_count {
                    self.current_addr
                        .store(self.regions[reg_idx].start, Ordering::Release);
                }
            }
        }

        None // Out of physical memory
    }

    unsafe fn deallocate_frame_internal(&self, frame: PhysFrame<Size4KiB>, phys_mem_offset: u64) {
        let phys_addr = frame.start_address().as_u64();
        let virt_ptr = (phys_addr + phys_mem_offset) as *mut u64;

        let mut current_head = self.head.load(Ordering::Relaxed);
        loop {
            unsafe { ptr::write_volatile(virt_ptr, current_head) };

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

impl FrameDeallocator<Size4KiB> for AtomicFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        unsafe {
            self.deallocate_frame_internal(frame, *PHYS_MEM_OFFSET.get());
        }
    }
}

#[derive(Copy, Clone)]
struct PhysRegion {
    start: u64,
    end: u64,
}
