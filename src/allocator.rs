use core::alloc::{GlobalAlloc, Layout};
use kernel_core::api;

#[global_allocator]
pub static ALLOC: Allocator = Allocator;

pub struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { api::memory().alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { api::memory().dealloc(ptr, layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        unsafe { api::memory().alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        unsafe { api::memory().realloc(ptr, layout, new_size) }
    }
}
