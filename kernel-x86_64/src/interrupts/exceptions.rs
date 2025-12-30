use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};

#[cold]
pub extern "x86-interrupt" fn x87_floating_point_handler(frame: InterruptStackFrame) {
    log::error!("Encountered x87 Floating Point Exception");
    log::error!("x87 Floating Point Exception: {:#?}", frame);
}

#[cold]
pub extern "x86-interrupt" fn vmm_communication_exception_handler(
    frame: InterruptStackFrame,
    code: u64,
) {
    log::error!("Encountered VMM Communication Exception");
    log::error!(
        "VMM Communication Exception with code {}: {:#?}",
        code,
        frame
    );
}

#[cold]
pub extern "x86-interrupt" fn virtualization_exception_handler(frame: InterruptStackFrame) {
    log::error!("Encountered Virtualization Exception");
    log::error!("Virtualization Exception: {:#?}", frame);
}

#[cold]
pub extern "x86-interrupt" fn stack_segment_fault_handler(frame: InterruptStackFrame, code: u64) {
    log::error!("Encountered Stack Segment Fault Exception");
    log::error!(
        "Stack Segment Fault Exception with code {}: {:#?}",
        code,
        frame
    );
}

#[cold]
pub extern "x86-interrupt" fn simd_floating_point_handler(frame: InterruptStackFrame) {
    log::error!("Encountered SIMD Floating Point Exception");
    log::error!("SIMD Floating Point Exception: {:#?}", frame);
}

#[cold]
pub extern "x86-interrupt" fn segment_not_present_handler(frame: InterruptStackFrame, code: u64) {
    log::error!("Encountered Segment Not Present Exception");
    log::error!(
        "Segment Not Present Exception with code {}: {:#?}",
        code,
        frame
    );
}

#[cold]
pub extern "x86-interrupt" fn security_exception_handler(frame: InterruptStackFrame, code: u64) {
    log::error!("Encountered Security Exception");
    log::error!("Security Exception with code {}: {:#?}", code, frame);
}

#[cold]
pub extern "x86-interrupt" fn page_fault_handler(
    frame: InterruptStackFrame,
    code: PageFaultErrorCode,
) {
    log::error!("Encountered Page Fault Exception");
    panic!("Page Fault Exception with code {:?}: {:#?}", code, frame);
}

#[cold]
pub extern "x86-interrupt" fn non_maskable_interrupt_handler(frame: InterruptStackFrame) {
    log::error!("Encountered Non Maskable Interrupt Exception");
    log::error!("Non Maskable Interrupt Exception: {:#?}", frame);
}

#[cold]
pub extern "x86-interrupt" fn machine_check_handler(frame: InterruptStackFrame) -> ! {
    log::error!("Encountered Machine Check Exception");
    panic!("Machine Check Exception: {:#?}", frame);
}

#[cold]
pub extern "x86-interrupt" fn invalid_tss_handler(frame: InterruptStackFrame, code: u64) {
    log::error!("Encountered Invalid TSS Exception");
    log::error!("Invalid TSS Exception with code {}: {:#?}", code, frame);
}

#[cold]
pub extern "x86-interrupt" fn invalid_opcode_handler(frame: InterruptStackFrame) {
    log::error!("Encountered Invalid Opcode Exception");
    log::error!("Invalid Opcode Exception: {:#?}", frame);
}

#[cold]
pub extern "x86-interrupt" fn hv_injection_exception_handler(frame: InterruptStackFrame) {
    log::error!("Encountered Hyper-V Injection Exception");
    log::error!("Hyper-V Injection Exception: {:#?}", frame);
}

#[cold]
pub extern "x86-interrupt" fn general_protection_fault_handler(
    frame: InterruptStackFrame,
    code: u64,
) {
    log::error!("Encountered General Protection Fault Exception");
    log::error!(
        "General Protection Fault Exception with code {}: {:#?}",
        code,
        frame
    );
}

#[cold]
pub extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, code: u64) -> ! {
    log::error!("Encountered Double Fault Exception");
    panic!("Double Fault Exception with code {}: {:#?}", code, frame);
}

#[cold]
pub extern "x86-interrupt" fn divide_error_handler(frame: InterruptStackFrame) {
    log::error!("Encountered Divide Error Exception");
    log::error!("Divide Error Exception: {:#?}", frame);
}

#[cold]
pub extern "x86-interrupt" fn device_not_available_handler(frame: InterruptStackFrame) {
    log::error!("Encountered Device Not Available Exception");
    log::error!("Device Not Available Exception: {:#?}", frame);
}

#[cold]
pub extern "x86-interrupt" fn cp_protection_exception_handler(
    frame: InterruptStackFrame,
    code: u64,
) {
    log::error!("Encountered CPU Protection Exception");
    log::error!("CPU Protection Exception with code {}: {:#?}", code, frame);
}

#[cold]
pub extern "x86-interrupt" fn breakpoint_handler(frame: InterruptStackFrame) {
    log::error!("Encountered Breakpoint Exception");
    log::error!("Breakpoint Exception: {:#?}", frame);
}

#[cold]
pub extern "x86-interrupt" fn alignment_check_handler(frame: InterruptStackFrame, code: u64) {
    log::error!("Encountered Alignment Check Exception");
    log::error!("Alignment Check Exception with code {}: {:#?}", code, frame);
}

#[cold]
pub extern "x86-interrupt" fn bound_range_exceeded_handler(frame: InterruptStackFrame) {
    log::error!("Encountered Bound Range Exceeded Exception");
    log::error!("Bound Range Exceeded Exception: {:#?}", frame);
}

#[cold]
pub extern "x86-interrupt" fn overflow_handler(frame: InterruptStackFrame) {
    log::error!("Encountered Overflow Exception");
    log::error!("Overflow Exception: {:#?}", frame);
}

#[cold]
pub extern "x86-interrupt" fn debug_handler(frame: InterruptStackFrame) {
    log::error!("Encountered Debug Exception");
    log::error!("Debug Exception: {:#?}", frame);
}
