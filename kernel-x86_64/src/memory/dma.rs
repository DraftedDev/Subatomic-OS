use crate::memory::{frame_alloc, phys_mem_offset};
use x86_64::PhysAddr;
use x86_64::structures::paging::{FrameAllocator, FrameDeallocator, PhysFrame, Size4KiB};

/// Allocates a DMA page.
///
/// # Safety
///
/// The caller must ensure that the allocated memory is used correctly
/// and that the physical address is valid for DMA operations.
pub unsafe fn dma_alloc(pages: usize) -> Option<(usize, usize)> {
    if pages == 0 {
        return None;
    }

    let offset = phys_mem_offset();

    if pages == 1 {
        let frame = frame_alloc::get().allocate_frame()?;
        let p_addr = frame.start_address().as_u64() as usize;
        let v_addr = p_addr + offset as usize;

        // Zero out allocated memory
        unsafe {
            core::ptr::write_bytes(v_addr as *mut u8, 0, 4096);
        }

        return Some((p_addr, v_addr));
    }

    allocate_contiguous_dma(pages, offset)
}

/// Deallocates a DMA page.
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

fn allocate_contiguous_dma(pages: usize, offset: u64) -> Option<(usize, usize)> {
    let alloc = frame_alloc::get();
    let mut frames = alloc::vec::Vec::with_capacity(pages);

    for _ in 0..pages {
        if let Some(frame) = alloc.allocate_frame() {
            frames.push(frame);
        } else {
            // Out of memory: rollback
            for f in frames {
                unsafe { alloc.deallocate_frame(f) };
            }
            return None;
        }
    }

    // Check physical contiguity
    let mut contiguous = true;
    for i in 0..pages - 1 {
        if frames[i + 1].start_address().as_u64() != frames[i].start_address().as_u64() + 4096 {
            contiguous = false;
            break;
        }
    }

    if contiguous {
        let p_addr = frames[0].start_address().as_u64() as usize;
        let v_addr = p_addr + offset as usize;
        unsafe {
            core::ptr::write_bytes(v_addr as *mut u8, 0, pages * 4096);
        }
        Some((p_addr, v_addr))
    } else {
        // Deallocate non-contiguous frames and return error
        for f in frames {
            unsafe { alloc.deallocate_frame(f) };
        }

        None
    }
}
