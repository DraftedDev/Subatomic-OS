use crate::acpi::ACPI;
use crate::interrupts::{apic, idt};
use crate::memory::{allocator, frame_alloc, mapper};
use crate::{cpuid, gdt, memory};
use kernel_core::device::{DeviceHub, pci};
use x86_64::PhysAddr;
use x86_64::structures::paging::PageTableFlags;

pub unsafe fn init() {
    unsafe {
        log::info!("Initializing CpuId...");
        cpuid::init();

        log::info!("Initializing Global Descriptor Table...");
        gdt::init();

        log::info!("Initializing Interrupt Descriptor Table...");
        idt::init();

        log::info!("Initializing physical memory offset...");
        memory::init_phys_mem();

        log::info!("Initializing page frame allocator...");
        frame_alloc::FRAME_ALLOCATOR.run(|alloc| alloc.init());

        log::info!("Initializing page mapper...");
        mapper::init();

        log::info!("Initializing heap allocator...");
        allocator::init();
    }
}

pub unsafe fn setup() {
    unsafe {
        log::info!("Initializing Advanced Configuration and Power Interface...");
        crate::acpi::init();

        log::info!("Initializing Advanced Programmable Interrupt Controller...");
        apic::init();

        log::info!("Initializing PCI Device Hub...");
        {
            let mcfg = ACPI.get().mcfg.get();
            let mcfg = mcfg.entries().first().expect("Failed to get MCFG");

            let mapped = mapper::map_address_range(
                PhysAddr::new(mcfg.base_address),
                (mcfg.bus_number_end as usize - mcfg.bus_number_start as usize + 1) * 0x100000, // 1MB per bus
                PageTableFlags::PRESENT | PageTableFlags::NO_CACHE | PageTableFlags::WRITABLE,
            );

            pci::init(mapped.as_u64() as usize)
                .run_mut(|hub| hub.init())
                .expect("Failed to initialize PCI Hub");
        }
    }
}
