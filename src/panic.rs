use alloc::string::ToString;
use core::panic::{Location, PanicInfo, PanicMessage};
use kernel_core::collections::{StackString, stack_format};
use kernel_core::{api, serial};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let location = info.location().unwrap();
    let msg = info.message();

    serial_panic(location, &msg);

    log::error!("Encountered kernel panic!");

    if api::memory().is_init() {
        log::error!(
            "Kernel Panic at {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
        log::error!("{}", msg.to_string());
    } else {
        log::error!("Kernel Panic at <location-not-available>");
        log::error!("{}", msg.as_str().unwrap_or("<HEAP NOT INITIALIZED>"));
    }

    loop {
        api::halt()
    }
}

fn serial_panic(loc: &Location, msg: &PanicMessage) {
    let location_str: StackString<16> = stack_format!("{}:{}", loc.line(), loc.column())
        .unwrap_or(StackString::try_from("?").unwrap());

    serial::print("KERNEL PANIC at ");
    serial::print(loc.file());
    serial::print(":");
    serial::print(location_str);
    serial::print("\n");

    if api::memory().is_init() {
        serial::println(msg.to_string());
    } else {
        serial::println(msg.as_str().unwrap_or("<FAILED TO GET ERROR MESSAGE>"));
    }
}
