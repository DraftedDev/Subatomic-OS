use crate::interrupts::{apic, idt};
use crate::memory::{allocator, frame_alloc, mapper};
use crate::{cpuid, gdt, memory};

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
    }
}
