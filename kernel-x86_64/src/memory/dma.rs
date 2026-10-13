use crate::memory::allocator::PAGE_SIZE;
use crate::memory::{self, frame_alloc};
use x86_64::PhysAddr;
use x86_64::structures::paging::PhysFrame;
use x86_64::structures::paging::{FrameAllocator, FrameDeallocator, Size4KiB};

/// Allocates a DMA page or contiguous block of pages and maps them into the page table.
///
/// # Safety
///
/// The caller must ensure that the allocated memory is used correctly
/// and that the physical address is valid for DMA operations.
pub unsafe fn dma_alloc(pages: usize) -> Option<(usize, usize)> {
    if pages == 0 {
        return None;
    }

    let alloc = frame_alloc::get();
    let size = pages * PAGE_SIZE;
    let offset = memory::phys_mem_offset();

    let first_frame = if pages == 1 {
        alloc.allocate_frame()?
    } else {
        alloc.allocate_contiguous(pages)?
    };

    let p_start = first_frame.start_address();
    let p_usize = p_start.as_u64() as usize;
    let v_usize = p_usize + offset as usize;

    // TODO: This really necessary?
    unsafe {
        core::ptr::write_bytes(v_usize as *mut u8, 0, size);
    }

    Some((p_usize, v_usize))
}

/// Deallocates a DMA page range.
///
/// # Safety
///
/// The caller must ensure that the physical address
/// and number of pages are valid and correspond to previously allocated DMA memory.
pub unsafe fn dma_dealloc(p_addr: usize, pages: usize) {
    let alloc = frame_alloc::get();

    for i in 0..pages {
        let frame_p_addr = PhysAddr::new((p_addr + i * 4096) as u64);
        let frame = PhysFrame::<Size4KiB>::containing_address(frame_p_addr);

        unsafe {
            alloc.deallocate_frame(frame);
        }
    }
}
