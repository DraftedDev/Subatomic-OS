use crate::acpi::ACPI;
use crate::interrupts::{apic, idt};
use crate::memory::{allocator, frame_alloc, mapper};
use crate::{cpuid, gdt, memory};
use kernel_core::device::usb;
use kernel_core::device::{DeviceHub, pci};
use x86_64::PhysAddr;
use x86_64::structures::paging::PageTableFlags;

/// Initialize the kernel.
///
/// The global control and display are not initialized at this point.
///
/// # Safety
/// Must only be called once before any initialization resources are used.
pub unsafe fn init() {
    unsafe {
        log::info!("Initializing CpuId...");
        cpuid::init();

        log::info!("Initializing Global Descriptor Table...");
        gdt::init();

        log::info!("Initializing Interrupt Descriptor Table...");
        idt::init();

        log::info!("Initializing Physical Memory Offset...");
        memory::init_phys_mem();

        log::info!("Initializing Page Frame Allocator...");
        frame_alloc::get().init();

        log::info!("Initializing Page Mapper...");
        mapper::init();

        log::info!("Initializing Heap Allocator...");
        allocator::init();
    }
}

/// Sets up the kernel after initialization.
///
/// The global control and display are initialized at this point.
///
/// # Safety
/// Must only be called once before any of the setup resources are used.
pub unsafe fn setup() {
    unsafe {
        let control = {
            log::info!("Registering architecture specific commands...");
            let control = kernel_core::control::CONTROL.get_mut();

            for command in crate::commands::COMMANDS {
                control.register(command);
            }

            control
        };

        control.update();

        log::info!("Initializing Advanced Configuration and Power Interface...");
        crate::acpi::init();

        control.update();

        log::info!("Initializing Advanced Programmable Interrupt Controller...");
        apic::init();

        log::info!("Initializing USB Device Hub...");
        usb::init();

        control.update();

        log::info!("Initializing PCI Device Hub...");
        {
            let mcfg = ACPI.get_unsafe().mcfg.get();
            let mcfg = mcfg.entries().first().expect("Failed to get MCFG");

            let mapped = mapper::map_address_range(
                PhysAddr::new(mcfg.base_address),
                (mcfg.bus_number_end as usize - mcfg.bus_number_start as usize + 1) * 0x100000, // 1MB per bus
                PageTableFlags::PRESENT | PageTableFlags::NO_CACHE | PageTableFlags::WRITABLE,
            );

            pci::init(mapped.as_u64() as usize)
                .run_mut(|hub| hub.init())
                .expect("Failed to initialize PCI Hub");

            log::info!("Registering built-in PCI drivers...");
            pci::add_builtin_drivers().expect("Failed to add built-in PCI drivers");
        }

        control.update();

        log::info!("Registering built-in USB drivers...");
        usb::add_builtin_drivers().expect("Failed to add built-in USB drivers");
    }
}
