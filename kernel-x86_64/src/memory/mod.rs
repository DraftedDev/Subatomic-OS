use kernel_core::requests;
use kernel_core::sync::init::InitData;

pub mod allocator;
pub mod frame_alloc;
pub mod mapper;

static PHYS_MEM_OFFSET: InitData<u64> = InitData::uninit();

/// Initialize the physical memory offset.
///
/// # Safety
/// This must only be called once before any physical memory translation operations.
pub unsafe fn init_phys_mem() {
    let hhdm = requests::higher_half_dm();

    unsafe {
        PHYS_MEM_OFFSET.init(hhdm.offset());
    }
}

pub const fn phys_mem_offset() -> u64 {
    *PHYS_MEM_OFFSET.get()
}
