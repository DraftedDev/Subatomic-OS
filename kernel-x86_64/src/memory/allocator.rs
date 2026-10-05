use crate::memory::frame_alloc;
use crate::memory::mapper::MAPPER;
use core::alloc::Layout;
use core::cmp::Ordering;
use core::ptr;
use kernel_core::sync::init::InitData;
use kernel_core::sync::mutex::Mutex;
use talc::base::Talc;
use x86_64::VirtAddr;
use x86_64::structures::paging::page::PageRangeInclusive;
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags};

pub const HEAP_START: usize = 0xffff_8800_0000_0000;
pub const HEAP_SIZE: usize = 16 * 1024 * 1024; // 16 MB

static ALLOCATOR: Mutex<Talc<talc::source::Manual, talc::DefaultBinning>> =
    Mutex::new(Talc::new(talc::source::Manual));

static INIT: InitData<bool> = InitData::uninit();

/// Initialize the allocator.
///
/// # Safety
/// Must only be called once before any allocations.
pub unsafe fn init() {
    MAPPER.get().run_mut_irq(|mapper| {
        let page_range: PageRangeInclusive = {
            let heap_start = VirtAddr::new(HEAP_START as u64);
            let heap_end = heap_start + HEAP_SIZE as u64 - 1u64;
            let heap_start_page = Page::containing_address(heap_start);
            let heap_end_page = Page::containing_address(heap_end);
            Page::range_inclusive(heap_start_page, heap_end_page)
        };

        for page in page_range {
            unsafe {
                mapper
                    .map_to(
                        page,
                        frame_alloc::get()
                            .allocate_frame()
                            .expect("failed to allocate frame"),
                        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                        frame_alloc::get(),
                    )
                    .expect("failed to map page frame")
                    .flush();
            };
        }

        ALLOCATOR.run(|talc| unsafe {
            talc.claim(HEAP_START as *mut u8, HEAP_SIZE)
                .expect("Failed to claim memory");
        });
    });

    unsafe {
        INIT.init(true);
    }
}

pub const fn is_init() -> bool {
    *INIT.get()
}

/// See [core::alloc::GlobalAlloc::alloc].
///
/// # Safety
/// The specified layout must be correct.
pub unsafe fn alloc(layout: Layout) -> *mut u8 {
    ALLOCATOR
        .run_irq(|talc| unsafe { talc.allocate(layout) })
        .map_or(ptr::null_mut(), |ptr| ptr.as_ptr())
}

/// See [core::alloc::GlobalAlloc::alloc_zeroed].
///
/// # Safety
/// The specified layout must be correct.
pub unsafe fn alloc_zeroed(layout: Layout) -> *mut u8 {
    // Copied from `GlobalAlloc`.
    ALLOCATOR.run_irq(|talc| unsafe {
        let size = layout.size();
        let ptr = talc
            .allocate(layout)
            .map_or(ptr::null_mut(), |ptr| ptr.as_ptr());

        if !ptr.is_null() {
            ptr::write_bytes(ptr, 0, size);
        }

        ptr
    })
}

/// See [core::alloc::GlobalAlloc::dealloc].
///
/// # Safety
/// The specified layout and pointer must be correct.
pub unsafe fn dealloc(ptr: *mut u8, layout: Layout) {
    ALLOCATOR.run_irq(|talc| unsafe { talc.deallocate(ptr, layout) })
}

/// See [core::alloc::GlobalAlloc::realloc].
///
/// # Safety
/// The specified layout, pointer and new size must be correct.
pub unsafe fn realloc(ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
    // Copied from `Talck`.
    ALLOCATOR.run_irq(|talc| unsafe {
        match new_size.cmp(&layout.size()) {
            Ordering::Greater => {
                if talc.try_grow_in_place(ptr, layout, new_size) {
                    return ptr;
                } // TODO: Else?

                let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());

                let allocation = match talc.allocate(new_layout) {
                    Some(ptr) => ptr,
                    None => return ptr::null_mut(),
                };

                allocation
                    .as_ptr()
                    .copy_from_nonoverlapping(ptr, layout.size());

                talc.deallocate(ptr, layout);

                allocation.as_ptr()
            }

            Ordering::Less => {
                talc.shrink(ptr, layout, new_size);
                ptr
            }

            Ordering::Equal => ptr,
        }
    })
}
