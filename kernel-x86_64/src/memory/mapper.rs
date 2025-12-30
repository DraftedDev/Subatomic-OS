use crate::memory::frame_alloc::FRAME_ALLOCATOR;
use crate::memory::phys_mem_offset;
use kernel_core::sync::init::InitData;
use kernel_core::sync::rwlock::RwLock;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{
    Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB, Translate,
};
use x86_64::{PhysAddr, VirtAddr};

pub type PageSize = Size4KiB;

pub static MAPPER: InitData<RwLock<OffsetPageTable<'static>>> = InitData::uninit();

pub unsafe fn init() {
    let phys_mem_offset = VirtAddr::new(phys_mem_offset());

    // get the active level 4 paging table
    let table = unsafe {
        let (level_4_table_frame, _) = Cr3::read();

        let phys = level_4_table_frame.start_address();
        let virt = phys_mem_offset + phys.as_u64();
        let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

        &mut *page_table_ptr
    };

    unsafe {
        MAPPER.init(RwLock::new(OffsetPageTable::new(table, phys_mem_offset)));
    }
}

pub fn translate_addr(addr: VirtAddr) -> Option<PhysAddr> {
    MAPPER.get().run(|table| table.translate_addr(addr))
}

pub fn translate_phys_addr(addr: PhysAddr) -> Option<VirtAddr> {
    let virt_addr = VirtAddr::new(addr.as_u64() + phys_mem_offset());

    // check if address is actually valid
    if translate_addr(virt_addr).is_some() {
        Some(virt_addr)
    } else {
        None
    }
}

pub unsafe fn translate_phys_addr_unsafe(addr: PhysAddr) -> VirtAddr {
    VirtAddr::new(addr.as_u64() + phys_mem_offset())
}

pub unsafe fn map_address(phys_addr: PhysAddr, flags: PageTableFlags) -> VirtAddr {
    FRAME_ALLOCATOR.run(|frame_alloc| {
        MAPPER.get().run_mut(|mapper| unsafe {
            let virt_addr = translate_phys_addr_unsafe(phys_addr);

            mapper
                .map_to(
                    Page::<PageSize>::containing_address(virt_addr),
                    PhysFrame::containing_address(phys_addr),
                    flags,
                    frame_alloc,
                )
                .expect("address mapping failed")
                .flush();

            virt_addr
        })
    })
}

pub unsafe fn map_address_if_not_present(phys_addr: PhysAddr, flags: PageTableFlags) -> VirtAddr {
    FRAME_ALLOCATOR.run(|frame_alloc| {
        MAPPER.get().run_mut(|mapper| unsafe {
            let virt_addr = translate_phys_addr_unsafe(phys_addr);

            if mapper.translate_addr(virt_addr).is_none() {
                mapper
                    .map_to(
                        Page::<PageSize>::containing_address(virt_addr),
                        PhysFrame::containing_address(phys_addr),
                        flags,
                        frame_alloc,
                    )
                    .expect("address mapping failed")
                    .flush();
            }

            virt_addr
        })
    })
}

pub unsafe fn map_address_range(start: PhysAddr, size: usize, flags: PageTableFlags) -> VirtAddr {
    FRAME_ALLOCATOR.run(|frame_alloc| {
        MAPPER.get().run_mut(|mapper| unsafe {
            // Round start and end addresses to 4KiB pages
            let page_size = 4096;
            let start_addr = start.as_u64() & !(page_size as u64 - 1);
            let end_addr =
                (start.as_u64() + size as u64 + page_size as u64 - 1) & !(page_size as u64 - 1);

            let mut current_addr = start_addr;
            let virt_start = translate_phys_addr_unsafe(PhysAddr::new(current_addr));

            while current_addr < end_addr {
                let phys_frame = PhysFrame::containing_address(PhysAddr::new(current_addr));
                let virt_page = Page::<PageSize>::containing_address(translate_phys_addr_unsafe(
                    PhysAddr::new(current_addr),
                ));

                // map the page if not already mapped
                if mapper.translate_addr(virt_page.start_address()).is_none() {
                    mapper
                        .map_to(virt_page, phys_frame, flags, frame_alloc)
                        .expect("address mapping failed")
                        .flush();
                }

                current_addr += page_size as u64;
            }

            virt_start
        })
    })
}
