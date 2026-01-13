#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![no_std]
#![no_main]

extern crate alloc;

use kernel_core::control::CONTROL;
use kernel_core::control::display::{DISPLAY, Display};
use kernel_core::info::KernelInfo;
use kernel_core::requests::BASE_REVISION;
use kernel_core::{api, control, logger};
use log::LevelFilter;

pub mod allocator;
pub mod panic;

#[unsafe(no_mangle)]
unsafe extern "C" fn kernel_main() -> ! {
    unsafe {
        init(
            option_env!("LOG_LEVEL")
                .unwrap_or("info")
                .parse()
                .expect("Invalid log level"),
        )
    };

    let info = KernelInfo::fetch();
    log::info!(
        "Running Subatomic OS:\n\
\t\tkernel: v{}\n\
\t\t{}: v{}\n\
\t\t{}: v{}",
        env!("CARGO_PKG_VERSION"),
        info.core.package,
        info.core.version,
        info.api.package,
        info.api.version
    );

    log::info!("Kernel setup completed. Continuing...");
    print_intro();

    loop {
        CONTROL.get().update();
        api::halt();
    }
}

unsafe fn init(filter: LevelFilter) {
    #[cfg(target_arch = "x86_64")]
    let kernel = unsafe { api::set(kernel_x86_64::KERNEL_API) };

    unsafe {
        logger::init(filter);
    }

    log::info!("Checking limine base revision support...");
    assert!(BASE_REVISION.is_supported());

    api::disable_interrupts();

    log::info!("Initializing kernel...");
    unsafe {
        (kernel.init)();
    }

    log::info!("Initializing Display...");
    unsafe {
        DISPLAY.init(Display::new());
    }

    log::info!("Initializing Control...");
    unsafe {
        control::init();
    }

    api::enable_interrupts();

    log::info!("Setting up kernel...");
    unsafe {
        (kernel.setup)();
    }
}

fn print_intro() {
    log::info!(
        r#"Running Subatomic OS by Mikail Plotzky

                      @@
                   @@    @@@
                  @     @@@@@@
                 @       @@@@
        @@@@@@# @@             @@@@@@
      @         @ @@@    @@@%         @@
     @@        @@     @@@              @
      @        @          @@.          @
       @       @     @@@@    @@      @@
        @@     @   @@@@@@@@     @@  @@
          @@   @  @@@@@@@@@       @@
     @@@@   @@ @   @@@@@@@@     @@  @@
     @@@@@     @     @@@@    @@      @@
      @@@      @          @@.          @
     @@        @@     @@@    @         @
      @          @@@@       @@ %@@@@  @@
        @@@@@@# @@          @  @@@@@@
                 @         @@   @@@
                  @       @@
                   @@    @
                      @@

Type 'help' for help.
"#
    );
}
