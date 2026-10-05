use crate::memory::frame_alloc;
use crate::memory::mapper::MAPPER;
use core::alloc::{GlobalAlloc, Layout};
use core::cmp::{max, min};
use core::sync::atomic::{AtomicU64, Ordering};
use core::{array, ptr};
use kernel_core::sync::init::InitData;
use x86_64::VirtAddr;
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB};

pub const HEAP_START: usize = 0xffff_8800_0000_0000;
pub const HEAP_SIZE: usize = 16 * 1024 * 1024; // 16 MB
pub const PAGE_SIZE: usize = 4096;
pub const TOTAL_PAGES: usize = HEAP_SIZE / PAGE_SIZE;

const NUM_SLAB_CLASSES: usize = 9;
const SLAB_SIZES: [usize; NUM_SLAB_CLASSES] = [8, 16, 32, 64, 128, 256, 512, 1024, 2048];

pub static GLOBAL_ALLOCATOR: InitData<HybridKernelAllocator> = InitData::uninit();

static mut INIT: bool = false;

pub fn init() {
    unsafe {
        GLOBAL_ALLOCATOR.init(HybridKernelAllocator::new());
        INIT = true;
    }
}

pub fn is_init() -> bool {
    unsafe { INIT }
}

#[repr(C)]
struct SlabNode {
    next: *mut SlabNode,
}

struct LocklessSlab {
    block_size: usize,
    head: AtomicU64,
}

impl LocklessSlab {
    const fn new(block_size: usize) -> Self {
        Self {
            block_size,
            head: AtomicU64::new(0),
        }
    }

    /// Pop an available object node from the atomic freelist (O(1) lockless).
    pub fn pop(&self) -> Option<*mut u8> {
        let mut current = self.head.load(Ordering::Acquire);
        loop {
            if current == 0 {
                return None;
            }

            let node_ptr = current as *mut SlabNode;
            // Read the intrusive next pointer inside the block
            let next_ptr = unsafe { ptr::read_volatile(&(*node_ptr).next) as u64 };

            match self.head.compare_exchange_weak(
                current,
                next_ptr,
                Ordering::Release,
                Ordering::Acquire,
            ) {
                Ok(_) => return Some(node_ptr as *mut u8),
                Err(actual) => current = actual,
            }
        }
    }

    /// Push a returned memory block onto the atomic freelist (O(1) lockless).
    pub unsafe fn push(&self, ptr: *mut u8) {
        let node_ptr = ptr as *mut SlabNode;
        let mut current = self.head.load(Ordering::Relaxed);

        loop {
            unsafe {
                ptr::write_volatile(&mut (*node_ptr).next, current as *mut SlabNode);
            }

            match self.head.compare_exchange_weak(
                current,
                node_ptr as u64,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current = actual,
            }
        }
    }
}

/// A lockless page allocator that manages pages via a 4096-bit atomic bitmap.
struct LocklessPageAllocator {
    bitmap: [AtomicU64; TOTAL_PAGES / 64],
    mapped_bitmap: [AtomicU64; TOTAL_PAGES / 64],
}

impl LocklessPageAllocator {
    pub fn new() -> Self {
        Self {
            bitmap: array::from_fn(|_| AtomicU64::new(0)),
            mapped_bitmap: array::from_fn(|_| AtomicU64::new(0)),
        }
    }

    /// Allocate `count` contiguous 4KiB virtual pages atomically using bitwise operations.
    pub fn alloc_pages(&self, count: usize) -> Option<VirtAddr> {
        if count == 0 || count > TOTAL_PAGES {
            return None;
        }

        // Single page fast path via Atomic Bit Scan
        if count == 1 {
            for (chunk_idx, atomic_chunk) in self.bitmap.iter().enumerate() {
                let mut current = atomic_chunk.load(Ordering::Relaxed);
                loop {
                    if current == !0u64 {
                        break; // All 64 pages in this chunk are allocated
                    }

                    // Find first zero bit using Trailing Zeros count (x86_64 TZCNT)
                    let free_bit = (!current).trailing_zeros() as usize;
                    let mask = 1u64 << free_bit;

                    match atomic_chunk.compare_exchange_weak(
                        current,
                        current | mask,
                        Ordering::Acquire,
                        Ordering::Relaxed,
                    ) {
                        Ok(_) => {
                            let page_idx = chunk_idx * 64 + free_bit;
                            let virt_addr =
                                VirtAddr::new((HEAP_START + page_idx * PAGE_SIZE) as u64);
                            self.ensure_pages_mapped(page_idx, 1);
                            return Some(virt_addr);
                        }
                        Err(actual) => current = actual,
                    }
                }
            }
            return None;
        }

        // Multi-page contiguous search
        self.alloc_contiguous_pages(count)
    }

    fn alloc_contiguous_pages(&self, count: usize) -> Option<VirtAddr> {
        let mut start_page = 0;
        let mut consecutive = 0;

        for page_idx in 0..TOTAL_PAGES {
            let chunk_idx = page_idx / 64;
            let bit_idx = page_idx % 64;
            let is_allocated =
                (self.bitmap[chunk_idx].load(Ordering::Relaxed) & (1u64 << bit_idx)) != 0;

            if is_allocated {
                consecutive = 0;
                start_page = page_idx + 1;
            } else {
                consecutive += 1;
                if consecutive == count {
                    // Attempt atomic reservation across contiguous range
                    if self.try_reserve_range(start_page, count) {
                        self.ensure_pages_mapped(start_page, count);
                        return Some(VirtAddr::new((HEAP_START + start_page * PAGE_SIZE) as u64));
                    } else {
                        // Retry search on collision
                        consecutive = 0;
                        start_page = page_idx + 1;
                    }
                }
            }
        }

        None
    }

    fn try_reserve_range(&self, start_page: usize, count: usize) -> bool {
        for page_idx in start_page..(start_page + count) {
            let chunk_idx = page_idx / 64;
            let bit_idx = page_idx % 64;
            let mask = 1u64 << bit_idx;

            let prev = self.bitmap[chunk_idx].fetch_or(mask, Ordering::Acquire);
            if (prev & mask) != 0 {
                // Collision occurred; rollback allocated bits
                for rollback_idx in start_page..page_idx {
                    let r_chunk = rollback_idx / 64;
                    let r_bit = rollback_idx % 64;
                    self.bitmap[r_chunk].fetch_and(!(1u64 << r_bit), Ordering::Release);
                }
                return false;
            }
        }
        true
    }

    pub fn free_pages(&self, virt_addr: VirtAddr, count: usize) {
        let page_idx = (virt_addr.as_u64() as usize - HEAP_START) / PAGE_SIZE;

        for i in 0..count {
            let idx = page_idx + i;
            let chunk_idx = idx / 64;
            let bit_idx = idx % 64;
            let mask = 1u64 << bit_idx;

            self.bitmap[chunk_idx].fetch_and(!mask, Ordering::Release);
        }
    }

    fn ensure_pages_mapped(&self, start_page: usize, count: usize) {
        MAPPER.get().run_mut_irq(|mapper| {
            for i in 0..count {
                let idx = start_page + i;
                let chunk_idx = idx / 64;
                let bit_idx = idx % 64;
                let mask = 1u64 << bit_idx;

                let was_mapped =
                    (self.mapped_bitmap[chunk_idx].fetch_or(mask, Ordering::AcqRel) & mask) != 0;

                if !was_mapped {
                    let page_addr = VirtAddr::new((HEAP_START + idx * PAGE_SIZE) as u64);
                    let page: Page<Size4KiB> = Page::containing_address(page_addr);

                    let frame = frame_alloc::get()
                        .allocate_frame()
                        .expect("Kernel Heap: Out of physical memory frames");

                    unsafe {
                        mapper
                            .map_to(
                                page,
                                frame,
                                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                                frame_alloc::get(),
                            )
                            .expect("Kernel Heap: Page mapping failure")
                            .flush();
                    }
                }
            }
        });
    }
}

/// A hybrid kernel allocator that combines slab allocation with a lockless page allocator.
pub struct HybridKernelAllocator {
    slabs: [LocklessSlab; NUM_SLAB_CLASSES],
    page_allocator: LocklessPageAllocator,
}

impl HybridKernelAllocator {
    pub fn new() -> Self {
        Self {
            slabs: [
                LocklessSlab::new(SLAB_SIZES[0]),
                LocklessSlab::new(SLAB_SIZES[1]),
                LocklessSlab::new(SLAB_SIZES[2]),
                LocklessSlab::new(SLAB_SIZES[3]),
                LocklessSlab::new(SLAB_SIZES[4]),
                LocklessSlab::new(SLAB_SIZES[5]),
                LocklessSlab::new(SLAB_SIZES[6]),
                LocklessSlab::new(SLAB_SIZES[7]),
                LocklessSlab::new(SLAB_SIZES[8]),
            ],
            page_allocator: LocklessPageAllocator::new(),
        }
    }

    fn get_slab_index(size: usize) -> Option<usize> {
        for (idx, &slab_size) in SLAB_SIZES.iter().enumerate() {
            if size <= slab_size {
                return Some(idx);
            }
        }
        None
    }

    /// Allocates memory using the hybrid kernel allocator.
    ///
    /// # Safety
    ///
    /// Caller must ensure that the layout is valid and the memory is initialized.
    pub unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let adjusted_size = max(layout.size(), layout.align());

        // Fast Path: Lockless Slab Engine
        if let Some(slab_idx) = Self::get_slab_index(adjusted_size) {
            let slab = &self.slabs[slab_idx];

            if let Some(ptr) = slab.pop() {
                return ptr;
            }

            // Expand slab pool by allocating a new 4KiB page from page allocator
            if let Some(page_addr) = self.page_allocator.alloc_pages(1) {
                let page_ptr = page_addr.as_u64() as *mut u8;
                let block_size = slab.block_size;
                let num_blocks = PAGE_SIZE / block_size;

                // Divide page into slab chunks and populate freelist
                for i in 1..num_blocks {
                    let block_ptr = unsafe { page_ptr.add(i * block_size) };
                    unsafe { slab.push(block_ptr) };
                }

                return page_ptr; // Return first block directly
            }

            return ptr::null_mut();
        }

        // Slow Path: Lockless Page Allocator for large objects (> 2048 Bytes)
        let required_pages = adjusted_size.div_ceil(PAGE_SIZE);
        match self.page_allocator.alloc_pages(required_pages) {
            Some(virt_addr) => virt_addr.as_u64() as *mut u8,
            None => ptr::null_mut(),
        }
    }

    /// Deallocates memory using the hybrid kernel allocator.
    ///
    /// # Safety
    ///
    /// Caller must ensure that the pointer is valid and the layout is correct.
    pub unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if ptr.is_null() {
            return;
        }

        let adjusted_size = max(layout.size(), layout.align());

        // Return object back to Lockless Slab
        if let Some(slab_idx) = Self::get_slab_index(adjusted_size) {
            unsafe { self.slabs[slab_idx].push(ptr) };
            return;
        }

        // Return page range back to Lockless Page Allocator
        let required_pages = adjusted_size.div_ceil(PAGE_SIZE);
        let virt_addr = VirtAddr::new(ptr as u64);
        self.page_allocator.free_pages(virt_addr, required_pages);
    }

    /// Reallocates memory using the hybrid kernel allocator.
    ///
    /// # Safety
    ///
    /// Caller must ensure that the pointer is valid and the layout is correct.
    pub unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if ptr.is_null() {
            let new_layout = match Layout::from_size_align(new_size, layout.align()) {
                Ok(l) => l,
                Err(_) => return ptr::null_mut(),
            };
            return unsafe { self.alloc(new_layout) };
        }

        if new_size == 0 {
            unsafe { self.dealloc(ptr, layout) };
            return ptr::null_mut();
        }

        let old_size = layout.size();

        // If new size fits in same slab class, reuse current block in place
        if let Some(old_slab) = Self::get_slab_index(old_size)
            && let Some(new_slab) = Self::get_slab_index(new_size)
            && old_slab == new_slab
        {
            return ptr;
        }

        let new_layout = match Layout::from_size_align(new_size, layout.align()) {
            Ok(l) => l,
            Err(_) => return ptr::null_mut(),
        };

        let new_ptr = unsafe { self.alloc(new_layout) };
        if !new_ptr.is_null() {
            let copy_size = min(old_size, new_size);
            unsafe { ptr::copy_nonoverlapping(ptr, new_ptr, copy_size) };
            unsafe { self.dealloc(ptr, layout) };
        }

        new_ptr
    }
}

unsafe impl GlobalAlloc for HybridKernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { self.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { self.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        unsafe { self.realloc(ptr, layout, new_size) }
    }
}
