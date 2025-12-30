use kernel_core::sync::init::InitData;
use raw_cpuid::{CpuId, CpuIdReaderNative};

static CPUID: InitData<CpuId<CpuIdReaderNative>> = InitData::uninit();

pub unsafe fn init() {
    unsafe {
        CPUID.init(CpuId::new());
    }
}

pub fn cpuid<'a>() -> &'a CpuId<CpuIdReaderNative> {
    CPUID.get()
}
