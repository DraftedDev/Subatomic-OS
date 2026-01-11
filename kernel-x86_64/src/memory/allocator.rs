use crate::memory::frame_alloc::FRAME_ALLOCATOR;
use crate::memory::mapper::MAPPER;
use core::alloc::Layout;
use core::cmp::Ordering;
use core::ptr;
use core::ptr::NonNull;
use kernel_core::sync::init::InitData;
use kernel_core::sync::mutex::Mutex;
use talc::{OomHandler, Span, Talc};
use x86_64::VirtAddr;
use x86_64::structures::paging::page::PageRangeInclusive;
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags};

pub const HEAP_START: usize = 0xffff_8800_0000_0000;
pub const HEAP_SIZE: usize = 16 * 1024 * 1024; // 16 MB

static ALLOCATOR: Mutex<Talc<OutOfMemory>> = Mutex::new(Talc::new(OutOfMemory));

static INIT: InitData<bool> = InitData::uninit();

/// Initialize the allocator.
///
/// # Safety
/// Must only be called once before any allocations.
pub unsafe fn init() {
    FRAME_ALLOCATOR.run(|frame_alloc| {
        MAPPER.get().run_mut(|mapper| {
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
                            frame_alloc
                                .allocate_frame()
                                .expect("failed to allocate frame"),
                            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                            frame_alloc,
                        )
                        .expect("failed to map page frame")
                        .flush();
                };
            }

            let span = Span::from_base_size(HEAP_START as *mut u8, HEAP_SIZE);

            ALLOCATOR.run(|talc| unsafe {
                talc.claim(span).expect("Failed to claim memory");
            });
        });
    });

    unsafe {
        INIT.init(true);
    }
}

pub const fn is_init() -> bool {
    *INIT.get()
}

pub fn alloc(layout: Layout) -> *mut u8 {
    ALLOCATOR
        .run(|talc| unsafe { talc.malloc(layout) })
        .map_or(ptr::null_mut(), |ptr| ptr.as_ptr())
}

pub fn alloc_zeroed(layout: Layout) -> *mut u8 {
    // Copied from `GlobalAlloc`.
    ALLOCATOR.run(|talc| unsafe {
        let size = layout.size();
        let ptr = talc
            .malloc(layout)
            .map_or(ptr::null_mut(), |ptr| ptr.as_ptr());

        if !ptr.is_null() {
            ptr::write_bytes(ptr, 0, size);
        }

        ptr
    })
}

pub fn dealloc(ptr: *mut u8, layout: Layout) {
    ALLOCATOR.run(|talc| unsafe { talc.free(NonNull::new_unchecked(ptr), layout) })
}

pub fn realloc(ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
    // Copied from `Talck`.
    ALLOCATOR.run(|talc| unsafe {
        const RELEASE_LOCK_ON_REALLOC_LIMIT: usize = 0x10000;

        let nn_ptr = NonNull::new_unchecked(ptr);

        match new_size.cmp(&layout.size()) {
            Ordering::Greater => {
                if let Ok(nn) = talc.grow_in_place(nn_ptr, layout, new_size) {
                    return nn.as_ptr();
                }

                let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());

                let allocation = match talc.malloc(new_layout) {
                    Ok(ptr) => ptr,
                    Err(_) => return ptr::null_mut(),
                };

                if layout.size() > RELEASE_LOCK_ON_REALLOC_LIMIT {
                    allocation
                        .as_ptr()
                        .copy_from_nonoverlapping(ptr, layout.size());
                } else {
                    allocation
                        .as_ptr()
                        .copy_from_nonoverlapping(ptr, layout.size());
                }

                talc.free(nn_ptr, layout);

                allocation.as_ptr()
            }

            Ordering::Less => {
                talc.shrink(NonNull::new_unchecked(ptr), layout, new_size);
                ptr
            }

            Ordering::Equal => ptr,
        }
    })
}

struct OutOfMemory;

impl OomHandler for OutOfMemory {
    fn handle_oom(_: &mut Talc<Self>, _: Layout) -> Result<(), ()> {
        log::error!("Out of memory!");

        Err(())
    }
}
