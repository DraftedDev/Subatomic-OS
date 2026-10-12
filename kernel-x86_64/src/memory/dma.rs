use crate::memory::{self, frame_alloc, mapper};
use x86_64::structures::paging::{
    FrameAllocator, FrameDeallocator, Page, PageTableFlags, Size4KiB,
};
use x86_64::structures::paging::{Mapper, PhysFrame};
use x86_64::{PhysAddr, VirtAddr};

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
    let size = pages * 4096;
    let offset = memory::phys_mem_offset();

    let first_frame = if pages == 1 {
        alloc.allocate_frame()?
    } else {
        alloc.allocate_contiguous(pages)?
    };

    let p_start = first_frame.start_address();
    let p_usize = p_start.as_u64() as usize;
    let v_usize = p_usize + offset as usize;

    let flags = PageTableFlags::PRESENT
        | PageTableFlags::WRITABLE
        | PageTableFlags::NO_CACHE
        | PageTableFlags::GLOBAL;

    mapper::MAPPER.get().run_mut(|m| {
        for i in 0..pages {
            let p_addr = p_start.as_u64() + (i as u64 * 4096);
            let v_addr = v_usize as u64 + (i as u64 * 4096);

            let frame = PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(p_addr));
            let page = Page::<Size4KiB>::containing_address(VirtAddr::new(v_addr));

            if let Ok(mapping) = unsafe { m.map_to(page, frame, flags, alloc) } {
                mapping.flush();
            } else if let Ok(mapping) = unsafe { m.update_flags(page, flags) } {
                mapping.flush();
            }
        }
    });

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
    let offset = crate::memory::phys_mem_offset();

    mapper::MAPPER.get().run_mut(|mapper| {
        for i in 0..pages {
            let frame_p_addr = PhysAddr::new((p_addr + i * 4096) as u64);
            let frame =
                x86_64::structures::paging::PhysFrame::<Size4KiB>::containing_address(frame_p_addr);
            let page = x86_64::structures::paging::Page::<Size4KiB>::containing_address(
                x86_64::VirtAddr::new(p_addr as u64 + offset + (i * 4096) as u64),
            );

            if let Ok((_frame, flush)) = mapper.unmap(page) {
                flush.flush();
            }

            unsafe {
                alloc.deallocate_frame(frame);
            }
        }
    });
}
