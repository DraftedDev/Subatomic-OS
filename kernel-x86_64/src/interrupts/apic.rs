use crate::acpi::{ACPI, HPET_CLOCK_TICK_UNIT, read_hpet_counter};
use crate::interrupts::{INTERRUPT_OFFSET, InterruptVector};
use crate::memory::mapper::map_address_if_not_present;
use acpi::sdt::madt::MadtEntry;
use kernel_core::sync::init::InitData;
use x2apic::ioapic::IoApic;
use x2apic::lapic::{LocalApic, LocalApicBuilder, TimerDivide, TimerMode};
use x86_64::PhysAddr;
use x86_64::instructions::port::Port;
use x86_64::structures::paging::PageTableFlags;

/// Milliseconds per APIC timer ticks.
///
/// Set to 10 millis for low overhead but frequently enough interrupts.
const MILLIS_PER_TICK: u32 = 10;

static IO_APIC: InitData<IoApic> = InitData::uninit();
static LOCAL_APIC: InitData<LocalApicWrapper> = InitData::uninit();

pub unsafe fn init() {
    // disable legacy PIC
    unsafe {
        Port::<u8>::new(0x21).write(0xff);
        Port::<u8>::new(0xa1).write(0xff);
    }

    // disable legacy PIT
    unsafe {
        let mut command_port: Port<u8> = Port::new(0x43);
        let mut channel_0_port: Port<u8> = Port::new(0x40);

        // Write the command to set Mode 0 (one-shot), Channel 0, Binary Counter
        command_port.write(0b00110000); // 00 (Channel 0), 11 (Access mode: LSB/MSB), 000 (Mode 0), 0 (Binary counter)
        channel_0_port.write(0); // Counter value = 0 to stop the timer
        channel_0_port.write(0); // Send MSB
    }

    let madt = ACPI.get().madt.get();

    let io_apic = madt
        .entries()
        .find_map(|entry| match entry {
            MadtEntry::IoApic(io_apic) => Some(io_apic),
            _ => None,
        })
        .expect("failed to find IoApic info");

    unsafe {
        init_io_apic(
            map_address_if_not_present(
                PhysAddr::new(io_apic.io_apic_address as u64),
                PageTableFlags::WRITABLE
                    | PageTableFlags::NO_CACHE
                    | PageTableFlags::PRESENT
                    | PageTableFlags::NO_EXECUTE,
            )
            .as_u64(),
        );

        init_local_apic(
            map_address_if_not_present(
                PhysAddr::new(x2apic::lapic::xapic_base()),
                PageTableFlags::WRITABLE
                    | PageTableFlags::NO_CACHE
                    | PageTableFlags::PRESENT
                    | PageTableFlags::NO_EXECUTE,
            )
            .as_u64(),
        );
    }
}

unsafe fn init_io_apic(apic_addr: u64) {
    unsafe {
        let mut apic = IoApic::new(apic_addr);

        apic.init(INTERRUPT_OFFSET);

        apic.enable_irq(InterruptVector::Keyboard as u8);

        IO_APIC.init(apic)
    };
}

unsafe fn init_local_apic(lapic_addr: u64) {
    let mut local_apic = LocalApicBuilder::new()
        .timer_divide(TimerDivide::Div16)
        .timer_initial(1_000_000)
        .timer_mode(TimerMode::Periodic)
        .timer_vector(InterruptVector::Timer.with_offset() as usize)
        .error_vector(InterruptVector::Error.with_offset() as usize)
        .spurious_vector(InterruptVector::Spurious.with_offset() as usize)
        .set_xapic_base(lapic_addr)
        .build()
        .expect("failed to create local apic");

    unsafe {
        local_apic.enable();

        crate::acpi::enable_hpet();

        calibrate_apic_timer(&mut local_apic);

        crate::acpi::disable_hpet();

        local_apic.enable_timer();

        LOCAL_APIC.init(LocalApicWrapper(local_apic));
    }
}

pub unsafe fn calibrate_apic_timer(lapic: &mut LocalApic) {
    let hpet_start = read_hpet_counter();

    unsafe {
        lapic.set_timer_mode(TimerMode::OneShot);
        lapic.set_timer_divide(TimerDivide::Div16);
        lapic.set_timer_initial(0xFFFFFFFF);
    }

    // wait for 10_000 HPET ticks
    const CALIBRATION_TICKS: u64 = 10_000;

    while read_hpet_counter().wrapping_sub(hpet_start) < CALIBRATION_TICKS {}

    // calculate how many APIC ticks have elapsed
    let elapsed_apic_ticks = 0xFFFFFFFF - unsafe { lapic.timer_current() } as u64;

    // get the HPET tick unit (in femto seconds)
    let clock_tick_unit_fs = *HPET_CLOCK_TICK_UNIT.get();

    // convert HPET ticks into nanoseconds
    let hpet_ticks_ns = CALIBRATION_TICKS * clock_tick_unit_fs / 1_000_000;

    // calculate APIC ticks per nanosecond
    let apic_ticks_per_ns = elapsed_apic_ticks as f64 / hpet_ticks_ns as f64;

    // calculate APIC ticks needed for 1ms (1,000,000 nanoseconds)
    let apic_ticks_per_ms = (apic_ticks_per_ns * 1_000_000.0) as u32;

    unsafe {
        lapic.set_timer_mode(TimerMode::Periodic);
        lapic.set_timer_initial(apic_ticks_per_ms * MILLIS_PER_TICK);
    }
}

pub unsafe fn end_of_interrupt() {
    unsafe {
        LOCAL_APIC.get_mut().0.end_of_interrupt();
    }
}

pub struct LocalApicWrapper(pub LocalApic);

// TODO: make local apic CPU-local
unsafe impl Send for LocalApicWrapper {}
unsafe impl Sync for LocalApicWrapper {}
