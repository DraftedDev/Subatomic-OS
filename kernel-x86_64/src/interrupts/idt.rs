use crate::interrupts::{InterruptVector, exceptions, keyboard, timer};
use kernel_core::sync::init::InitData;
use x86_64::structures::idt::InterruptDescriptorTable;

static IDT: InitData<InterruptDescriptorTable> = InitData::uninit();

pub unsafe fn init() {
    let mut idt: InterruptDescriptorTable = InterruptDescriptorTable::new();

    idt.overflow.set_handler_fn(exceptions::overflow_handler);

    idt.debug.set_handler_fn(exceptions::debug_handler);

    idt.bound_range_exceeded
        .set_handler_fn(exceptions::bound_range_exceeded_handler);

    idt.alignment_check
        .set_handler_fn(exceptions::alignment_check_handler);

    idt.breakpoint
        .set_handler_fn(exceptions::breakpoint_handler);

    idt.cp_protection_exception
        .set_handler_fn(exceptions::cp_protection_exception_handler);

    idt.device_not_available
        .set_handler_fn(exceptions::device_not_available_handler);

    idt.divide_error
        .set_handler_fn(exceptions::divide_error_handler);

    idt.double_fault
        .set_handler_fn(exceptions::double_fault_handler);

    idt.general_protection_fault
        .set_handler_fn(exceptions::general_protection_fault_handler);

    idt.hv_injection_exception
        .set_handler_fn(exceptions::hv_injection_exception_handler);

    idt.invalid_opcode
        .set_handler_fn(exceptions::invalid_opcode_handler);

    idt.invalid_tss
        .set_handler_fn(exceptions::invalid_tss_handler);

    idt.machine_check
        .set_handler_fn(exceptions::machine_check_handler);

    idt.non_maskable_interrupt
        .set_handler_fn(exceptions::non_maskable_interrupt_handler);

    idt.page_fault
        .set_handler_fn(exceptions::page_fault_handler);

    idt.security_exception
        .set_handler_fn(exceptions::security_exception_handler);

    idt.segment_not_present
        .set_handler_fn(exceptions::segment_not_present_handler);

    idt.simd_floating_point
        .set_handler_fn(exceptions::simd_floating_point_handler);

    idt.stack_segment_fault
        .set_handler_fn(exceptions::stack_segment_fault_handler);

    idt.virtualization
        .set_handler_fn(exceptions::virtualization_exception_handler);

    idt.vmm_communication_exception
        .set_handler_fn(exceptions::vmm_communication_exception_handler);

    idt.x87_floating_point
        .set_handler_fn(exceptions::x87_floating_point_handler);

    idt[InterruptVector::Timer.with_offset()].set_handler_fn(timer::timer_handler);

    idt[InterruptVector::Keyboard.with_offset()]
        .set_handler_fn(keyboard::keyboard_interrupt_handler);

    unsafe { IDT.init(idt).load() }
}
