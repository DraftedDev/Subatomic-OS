use crate::memory::phys_mem_offset;
use core::ptr;
use kernel_core::requests;
use kernel_core::sync::mutex::Mutex;
use limine::memory_map::EntryType;
use x86_64::PhysAddr;
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};

/// Global frame allocator
pub static FRAME_ALLOCATOR: Mutex<PageFrameAllocator> = Mutex::new(PageFrameAllocator::new());

/// Intrusive linked list page frame allocator.
/// Free frames store a pointer to the next free frame in their first 8 bytes.
pub struct PageFrameAllocator {
    head: Option<PhysFrame<Size4KiB>>,
    free_count: usize,
}

impl PageFrameAllocator {
    pub const fn new() -> Self {
        Self {
            head: None,
            free_count: 0,
        }
    }

    /// Initialize the allocator from all usable memory regions.
    ///
    /// # Safety
    /// Must only be called once, before any allocation.
    pub unsafe fn init(&mut self) {
        let phys_mem_offset = phys_mem_offset();

        for region in requests::memory_map().entries() {
            // TODO: reclaim acpi/bootloader memory
            if region.entry_type != EntryType::USABLE {
                continue;
            }

            let start = region.base;
            let end = region.base + region.length;

            let mut addr = start;
            while addr < end {
                let frame = PhysFrame::containing_address(PhysAddr::new(addr));

                unsafe {
                    self.push_frame(frame, phys_mem_offset);
                }

                addr += 4096;
            }
        }
    }

    /// Push a frame onto the free list.
    unsafe fn push_frame(&mut self, frame: PhysFrame<Size4KiB>, phys_mem_offset: u64) {
        let phys = frame.start_address();
        let virt = (phys.as_u64() + phys_mem_offset) as *mut u64;

        // write the previous head into this frame
        let next_addr = match self.head {
            Some(f) => f.start_address().as_u64(),
            None => 0,
        };
        unsafe { ptr::write(virt, next_addr) };

        self.head = Some(frame);
        self.free_count += 1;
    }

    /// Pop a frame from the free list.
    unsafe fn pop_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let phys_mem_offset = phys_mem_offset();
        let head = self.head?;
        let virt = (head.start_address().as_u64() + phys_mem_offset) as *const u64;
        let next_phys = unsafe { ptr::read(virt) };

        self.head = if next_phys == 0 {
            None
        } else {
            Some(PhysFrame::containing_address(PhysAddr::new(next_phys)))
        };

        self.free_count = self.free_count.wrapping_sub(1);

        Some(head)
    }

    /// Number of free frames remaining.
    pub fn free_count(&self) -> usize {
        self.free_count
    }
}

unsafe impl FrameAllocator<Size4KiB> for PageFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        unsafe { self.pop_frame() }
    }
}
