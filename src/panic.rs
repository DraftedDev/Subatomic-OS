use core::panic::PanicInfo;
use kernel_core::api;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let location = info.location().unwrap();
    let msg = info.message();

    log::error!("Encountered kernel panic!");

    log::error!(
        "Kernel Panic at {}:{}:{}",
        location.file(),
        location.line(),
        location.column()
    );

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
