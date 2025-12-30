use alloc::string::String;

pub use no_pico_args as args;

/// A command that can be executed.
#[derive(Copy, Clone, Debug)]
pub struct Command {
    /// The name of the command.
    pub name: &'static str,
    /// The description of the command.
    pub description: &'static str,
    /// The usage of the command. Displayed inside the `help` message.
    pub usage: &'static str,
    /// The function to run when the command is executed.
    pub run: fn(String) -> Result<(), String>,
}

/// Built-in commands.
pub mod builtin {
    use crate::control::command::Command;
    use crate::info::KernelInfo;
    use crate::{api, requests};
    use alloc::format;
    use alloc::string::String;

    /// Built-in commands.
    pub const COMMANDS: &[Command] = &[
        Command {
            name: "sys-info",
            description: "Prints information about the system to the control.",
            usage: "sys-info",
            run: sys_info,
        },
        Command {
            name: "time",
            description: "Prints the current time to the control.",
            usage: "time",
            run: time,
        },
    ];

    fn sys_info(_: String) -> Result<(), String> {
        let info = KernelInfo::fetch();
        let bootloader = requests::bootloader_info();

        let info = format!(
            "Running SubatomicOS by Mikail Plotzky\n\
            \tBootloader: {} v{}\n\
        \t{} v{}\n\
        \t{} v{}",
            bootloader.name(),
            bootloader.version(),
            info.core.package,
            info.core.version,
            info.api.package,
            info.api.version,
        );

        log::info!("System information:\n{info}");

        Ok(())
    }

    fn time(_: String) -> Result<(), String> {
        let time = api::time().read();

        log::info!(
            "Time: secs({}) mins({}) hours({}) days({}) months({}) years({})",
            time.sec,
            time.min,
            time.hour,
            time.day,
            time.month,
            time.year,
        );

        Ok(())
    }
}
