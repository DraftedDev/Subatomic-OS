use crate::interrupts::apic;
use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn timer_handler(_: InterruptStackFrame) {
    unsafe {
        // TODO: handle time

        apic::end_of_interrupt();
    }
}
