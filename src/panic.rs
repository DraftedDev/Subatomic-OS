use core::panic::{Location, PanicInfo, PanicMessage};
use kernel_core::collections::{StackString, stack_format};
use kernel_core::{api, serial_println};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let location = info.location().unwrap();
    let msg = info.message();

    serial_panic(&msg, location);

    log::error!("Encountered kernel panic!");

    // TODO: handle this stack overflow
    let message: StackString<512> = stack_format!(
        "Kernel Panic at {}:{}:{}",
        location.file(),
        location.line(),
        location.column()
    )
    .unwrap();

    log::error!("{}", message);

    if api::memory().is_init() {
        log::error!("Panic Message: {}", msg);
    } else {
        log::error!(
            "Panic Message: {}",
            msg.as_str().unwrap_or("<HEAP NOT INITIALIZED>")
        );
    }

    loop {
        api::halt()
    }
}

// TODO: investigate into log::error not working somehow
fn serial_panic(msg: &PanicMessage, location: &Location) {
    serial_println!("Encountered kernel panic!");

    // TODO: handle this stack overflow
    let message: StackString<512> = stack_format!(
        "Kernel Panic at {}:{}:{}",
        location.file(),
        location.line(),
        location.column()
    )
    .unwrap();

    serial_println!("{}", message);

    if api::memory().is_init() {
        serial_println!("Panic Message: {}", msg);
    } else {
        serial_println!(
            "Panic Message: {}",
            msg.as_str().unwrap_or("<HEAP NOT INITIALIZED>")
        );
    }
}
