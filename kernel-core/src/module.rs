// TODO: docs
#![allow(missing_docs)]

use crate::sync::init::InitData;
use crate::{api, requests};
use alloc::vec::Vec;
use core::alloc::Layout;
use object::{File, Object, ObjectSegment, ObjectSymbol};

pub static MODULES: InitData<Vec<KernelModule>> = InitData::uninit();

pub unsafe fn init() {
    if let Some(response) = requests::modules() {
        let limine_modules = response.modules();
        let mut modules = Vec::with_capacity(limine_modules.len());

        for file in limine_modules {
            let module = KernelModule::load_limine(file);
            log::info!("Loaded limine module {}", module.name);
            modules.push(module);
        }

        unsafe { MODULES.init(modules) };
    } else {
        log::info!("No limine modules found.");
    }
}

pub fn run_init() {
    for module in MODULES.get() {
        module.init();
    }
}

pub fn run_update() {
    for module in MODULES.get() {
        module.update();
    }
}

#[derive(Copy, Clone, Debug)]
pub struct KernelModule {
    pub name: &'static str,
    pub version: &'static str,
    pub author: &'static str,
    pub description: &'static str,
    pub init: fn(),
    pub update: fn(),
}

impl KernelModule {
    pub fn load_limine(module: &limine::file::File) -> Self {
        log::info!(
            "Loading internal limine module {}...",
            module.path().to_string_lossy()
        );
        let addr = module.addr() as *const u8;
        let size = module.size() as usize;

        let bytes = unsafe { core::slice::from_raw_parts(addr, size) };

        KernelModule::load(bytes)
    }

    pub fn load(bytes: impl AsRef<[u8]>) -> KernelModule {
        let file = File::parse(bytes.as_ref()).expect("Failed to load module");

        // Allocate all PT_LOAD segments
        let mut module_base = 0;
        for segment in file.segments() {
            let size = segment.data().unwrap().len();
            let addr =
                unsafe { api::memory().alloc(Layout::from_size_align(size, 0x1000).unwrap()) };

            if module_base == 0 {
                module_base = addr as usize;
            }

            unsafe {
                core::ptr::copy_nonoverlapping(segment.data().unwrap().as_ptr(), addr, size);
            }
        }

        // Minimal relocations
        for (addr, rel) in file
            .dynamic_relocations()
            .expect("Failed to get dynamic relocations")
        {
            let target = module_base + addr as usize;
            let addend = rel.addend() as usize;

            match rel.kind() {
                object::RelocationKind::Absolute => unsafe {
                    *(target as *mut usize) = module_base + addend
                },
                _ => panic!("Unsupported relocation"),
            }
        }

        let module_symbol = file
            .symbols()
            .find(|s| s.name() == Ok("KERNEL_MODULE"))
            .expect("Failed to find 'KERNEL_MODULE' symbol");

        let module_ptr = (module_base + module_symbol.address() as usize) as *const KernelModule;

        unsafe { *module_ptr }
    }

    pub fn init(&self) {
        (self.init)()
    }

    pub fn update(&self) {
        (self.update)()
    }
}
