use kernel_core::serial_println;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};

#[cold]
pub extern "x86-interrupt" fn x87_floating_point_handler(frame: InterruptStackFrame) {
    serial_println!("\n=== EXCEPTION: x87 FLOATING POINT ===");
    serial_println!("Frame: {:#?}", frame);

    log::error!("Encountered x87 Floating Point Exception: {:#?}", frame);

    loop {
        kernel_core::api::kernel().halt();
    }
}

#[cold]
pub extern "x86-interrupt" fn vmm_communication_exception_handler(
    frame: InterruptStackFrame,
    code: u64,
) {
    serial_println!("\n=== EXCEPTION: VMM COMMUNICATION ===");
    serial_println!("Error Code: {:#x}", code);
    serial_println!("Frame     : {:#?}", frame);

    log::error!(
        "VMM Communication Exception with code {:#x}: {:#?}",
        code,
        frame
    );

    loop {
        kernel_core::api::kernel().halt();
    }
}

#[cold]
pub extern "x86-interrupt" fn virtualization_exception_handler(frame: InterruptStackFrame) {
    serial_println!("\n=== EXCEPTION: VIRTUALIZATION ===");
    serial_println!("Frame: {:#?}", frame);

    log::error!("Encountered Virtualization Exception: {:#?}", frame);

    loop {
        kernel_core::api::kernel().halt();
    }
}

#[cold]
pub extern "x86-interrupt" fn stack_segment_fault_handler(frame: InterruptStackFrame, code: u64) {
    serial_println!("\n=== EXCEPTION: STACK SEGMENT FAULT ===");
    serial_println!("Error Code: {:#x}", code);
    serial_println!("Frame     : {:#?}", frame);

    log::error!(
        "Stack Segment Fault Exception with code {:#x}: {:#?}",
        code,
        frame
    );

    loop {
        kernel_core::api::kernel().halt();
    }
}

#[cold]
pub extern "x86-interrupt" fn simd_floating_point_handler(frame: InterruptStackFrame) {
    serial_println!("\n=== EXCEPTION: SIMD FLOATING POINT ===");
    serial_println!("Frame: {:#?}", frame);

    log::error!("Encountered SIMD Floating Point Exception: {:#?}", frame);

    loop {
        kernel_core::api::kernel().halt();
    }
}

#[cold]
pub extern "x86-interrupt" fn segment_not_present_handler(frame: InterruptStackFrame, code: u64) {
    serial_println!("\n=== EXCEPTION: SEGMENT NOT PRESENT ===");
    serial_println!("Error Code: {:#x}", code);
    serial_println!("Frame     : {:#?}", frame);

    log::error!(
        "Segment Not Present Exception with code {:#x}: {:#?}",
        code,
        frame
    );

    loop {
        kernel_core::api::kernel().halt();
    }
}

#[cold]
pub extern "x86-interrupt" fn security_exception_handler(frame: InterruptStackFrame, code: u64) {
    serial_println!("\n=== EXCEPTION: SECURITY ===");
    serial_println!("Error Code: {:#x}", code);
    serial_println!("Frame     : {:#?}", frame);

    log::error!("Security Exception with code {:#x}: {:#?}", code, frame);

    loop {
        kernel_core::api::kernel().halt();
    }
}

#[cold]
pub extern "x86-interrupt" fn page_fault_handler(
    frame: InterruptStackFrame,
    code: PageFaultErrorCode,
) {
    let faulting_addr = Cr2::read();
    serial_println!("\n=== EXCEPTION: PAGE FAULT ===");
    serial_println!("Faulting Address (CR2): {:?}", faulting_addr);
    serial_println!("Error Code            : {:?}", code);
    serial_println!("Frame                 : {:#?}", frame);

    log::error!(
        "Page Fault at {:?} with code {:?}: {:#?}",
        faulting_addr,
        code,
        frame
    );

    loop {
        kernel_core::api::kernel().halt();
    }
}

#[cold]
pub extern "x86-interrupt" fn non_maskable_interrupt_handler(frame: InterruptStackFrame) {
    serial_println!("\n=== EXCEPTION: NON MASKABLE INTERRUPT ===");
    serial_println!("Frame: {:#?}", frame);

    log::error!("Encountered Non Maskable Interrupt Exception: {:#?}", frame);

    loop {
        kernel_core::api::kernel().halt();
    }
}

#[cold]
pub extern "x86-interrupt" fn machine_check_handler(frame: InterruptStackFrame) -> ! {
    serial_println!("\n=== EXCEPTION: MACHINE CHECK ===");
    serial_println!("Frame: {:#?}", frame);

    log::error!("Machine Check Exception: {:#?}", frame);

    loop {
        kernel_core::api::kernel().halt();
    }
}

#[cold]
pub extern "x86-interrupt" fn invalid_tss_handler(frame: InterruptStackFrame, code: u64) {
    serial_println!("\n=== EXCEPTION: INVALID TSS ===");
    serial_println!("Error Code: {:#x}", code);
    serial_println!("Frame     : {:#?}", frame);

    log::error!("Invalid TSS Exception with code {:#x}: {:#?}", code, frame);

    loop {
        kernel_core::api::kernel().halt();
    }
}

#[cold]
pub extern "x86-interrupt" fn invalid_opcode_handler(frame: InterruptStackFrame) {
    serial_println!("\n=== EXCEPTION: INVALID OPCODE ===");
    serial_println!("Frame: {:#?}", frame);

    log::error!("Encountered Invalid Opcode Exception: {:#?}", frame);

    loop {
        kernel_core::api::kernel().halt();
    }
}

#[cold]
pub extern "x86-interrupt" fn hv_injection_exception_handler(frame: InterruptStackFrame) {
    serial_println!("\n=== EXCEPTION: HYPER-V INJECTION ===");
    serial_println!("Frame: {:#?}", frame);

    log::error!("Encountered Hyper-V Injection Exception: {:#?}", frame);

    loop {
        kernel_core::api::kernel().halt();
    }
}

#[cold]
pub extern "x86-interrupt" fn general_protection_fault_handler(
    frame: InterruptStackFrame,
    code: u64,
) {
    serial_println!("\n=== EXCEPTION: GENERAL PROTECTION FAULT ===");
    serial_println!("Error Code: {:#x}", code);
    serial_println!("Frame     : {:#?}", frame);

    log::error!(
        "General Protection Fault Exception with code {:#x}: {:#?}",
        code,
        frame
    );

    loop {
        kernel_core::api::kernel().halt();
    }
}

#[cold]
pub extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, code: u64) -> ! {
    serial_println!("\n=== EXCEPTION: DOUBLE FAULT ===");
    serial_println!("Error Code: {:#x}", code);
    serial_println!("Frame     : {:#?}", frame);

    log::error!("Double Fault Exception with code {:#x}: {:#?}", code, frame);

    loop {
        kernel_core::api::kernel().halt();
    }
}

#[cold]
pub extern "x86-interrupt" fn divide_error_handler(frame: InterruptStackFrame) {
    serial_println!("\n=== EXCEPTION: DIVIDE ERROR ===");
    serial_println!("Frame: {:#?}", frame);

    log::error!("Encountered Divide Error Exception: {:#?}", frame);

    loop {
        kernel_core::api::kernel().halt();
    }
}

#[cold]
pub extern "x86-interrupt" fn device_not_available_handler(frame: InterruptStackFrame) {
    serial_println!("\n=== EXCEPTION: DEVICE NOT AVAILABLE ===");
    serial_println!("Frame: {:#?}", frame);

    log::error!("Encountered Device Not Available Exception: {:#?}", frame);

    loop {
        kernel_core::api::kernel().halt();
    }
}

#[cold]
pub extern "x86-interrupt" fn cp_protection_exception_handler(
    frame: InterruptStackFrame,
    code: u64,
) {
    serial_println!("\n=== EXCEPTION: CONTROL PROTECTION ===");
    serial_println!("Error Code: {:#x}", code);
    serial_println!("Frame     : {:#?}", frame);

    log::error!(
        "CPU Control Protection Exception with code {:#x}: {:#?}",
        code,
        frame
    );

    loop {
        kernel_core::api::kernel().halt();
    }
}

#[cold]
pub extern "x86-interrupt" fn breakpoint_handler(frame: InterruptStackFrame) {
    serial_println!("\n=== EXCEPTION: BREAKPOINT ===");
    serial_println!("Frame: {:#?}", frame);

    log::error!("Encountered Breakpoint Exception: {:#?}", frame);
}

#[cold]
pub extern "x86-interrupt" fn alignment_check_handler(frame: InterruptStackFrame, code: u64) {
    serial_println!("\n=== EXCEPTION: ALIGNMENT CHECK ===");
    serial_println!("Error Code: {:#x}", code);
    serial_println!("Frame     : {:#?}", frame);

    log::error!(
        "Alignment Check Exception with code {:#x}: {:#?}",
        code,
        frame
    );

    loop {
        kernel_core::api::kernel().halt();
    }
}

#[cold]
pub extern "x86-interrupt" fn bound_range_exceeded_handler(frame: InterruptStackFrame) {
    serial_println!("\n=== EXCEPTION: BOUND RANGE EXCEEDED ===");
    serial_println!("Frame: {:#?}", frame);

    log::error!("Encountered Bound Range Exceeded Exception: {:#?}", frame);

    loop {
        kernel_core::api::kernel().halt();
    }
}

#[cold]
pub extern "x86-interrupt" fn overflow_handler(frame: InterruptStackFrame) {
    serial_println!("\n=== EXCEPTION: OVERFLOW ===");
    serial_println!("Frame: {:#?}", frame);

    log::error!("Encountered Overflow Exception: {:#?}", frame);

    loop {
        kernel_core::api::kernel().halt();
    }
}

#[cold]
pub extern "x86-interrupt" fn debug_handler(frame: InterruptStackFrame) {
    serial_println!("\n=== EXCEPTION: DEBUG ===");
    serial_println!("Frame: {:#?}", frame);

    log::error!("Encountered Debug Exception: {:#?}", frame);

    loop {
        kernel_core::api::kernel().halt();
    }
}
