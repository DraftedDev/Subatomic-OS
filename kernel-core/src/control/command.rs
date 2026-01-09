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
    use crate::device::DeviceHub;
    use crate::info::KernelInfo;
    use crate::time::TimeZone;
    use crate::{api, requests};
    use alloc::format;
    use alloc::string::{String, ToString};
    use no_pico_args::Arguments;
    use time::UtcOffset;

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
            description: "Prints the current time to the control or sets the time zone.",
            usage: "time <local|utc|set <zone|+hh:+mm:+ss>|list>",
            run: time,
        },
        #[cfg(feature = "pci")]
        Command {
            name: "pci",
            description: "Prints information about the PCI devices to the control.",
            usage: "pci <info>",
            run: pci,
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

    fn time(sub: String) -> Result<(), String> {
        let mut args = Arguments::from_string(sub);

        if let Some(sub) = args.subcommand() {
            match sub.as_str() {
                sub if sub == "local" || sub == "utc" => {
                    let time = if sub == "local" {
                        api::time().read_local()
                    } else if sub == "utc" {
                        api::time().read_utc().to_offset(UtcOffset::UTC)
                    } else {
                        api::time().read_local()
                    };

                    log::info!(
                        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                        time.year(),
                        time.month() as u8,
                        time.day(),
                        time.hour(),
                        time.minute(),
                        time.second()
                    );
                }

                "set" => {
                    let arg = args
                        .subcommand()
                        .ok_or("Please specify a new time offset".to_string())?;

                    let zone =
                        TimeZone::parse(&arg).ok_or(format!("Invalid time zone: {}", arg))?;
                    let (hours, mins, secs) = zone.to_offset();
                    api::time().set_offset(hours, mins, secs);

                    log::info!(
                        "Time offset updated with hours({}) minutes({}) seconds({})",
                        hours,
                        mins,
                        secs
                    );
                }

                "list" => {
                    log::info!("Listing all named time zones:");

                    for zone in TimeZone::NAMED_ZONES {
                        log::info!("{} - {:?}", zone.as_symbol().unwrap(), zone);
                    }
                }

                "help" => log::info!(
                    "Print out the time or set the system time zone.\n\
            \tUsage: `time <local|utc|set <zone>|list>`\n\
            \tExample to get local time: `time local`\n\
            \tExample to set the time zone to EST: `time set EST`\n\
            \tExample to set the time zone to 10:30:00: `time set +10:+30:+00`\n\
            \tExample to list all available time zones: `time list`"
                ),

                _ => {
                    return Err(format!(
                        "Invalid subcommand: {}. Usage: `time <local|utc|set <zone>|list>`.",
                        sub
                    ));
                }
            }
        } else {
            return Err(
                "No subcommand specified. Usage: `time <local|utc|set <zone>|list>`.".to_string(),
            );
        }

        Ok(())
    }

    #[cfg(feature = "pci")]
    fn pci(sub: String) -> Result<(), String> {
        let mut args = Arguments::from_string(sub);

        if let Some(sub) = args.subcommand() {
            match sub.as_str() {
                "info" => {
                    api::without_interrupts(|| {
                        crate::device::pci::PCI_HUB.get().run(|hub| {
                            for (idx, dev) in hub.devices().iter().enumerate() {
                                let dev = hub.get(dev).expect("Failed to get device");
                                let class = dev.class();
                                let (ven_id, dev_id) = dev.id();

                                log::info!(
                                    "{idx}: Device at Address {}\n\
                            \t- Header Type: {:?}\n\
                            \t- Class: {:?}\n\
                            \t- Interface: {}\n\
                            \t- Revision: {}\n\
                            \t- Device/Vendor ID: {:?}/{:?}\n\
                            \t- Command: {:?}\n\
                            \t- Capabilities: {:?}",
                                    dev.addr(),
                                    dev.header_type(),
                                    class,
                                    dev.interface(),
                                    dev.revision(),
                                    ven_id,
                                    dev_id,
                                    dev.command(),
                                    dev.capabilities()
                                );
                            }
                        })
                    });
                }
                _ => return Err(format!("Invalid subcommand: {}. Usage: `pci <info>`.", sub)),
            }
        } else {
            return Err("No subcommand specified. Usage: `pci <info>`.".to_string());
        }

        Ok(())
    }
}
