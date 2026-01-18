use crate::cpuid;
use alloc::string::String;
use kernel_core::control::command::Command;

pub const COMMANDS: [Command; 1] = [Command {
    name: "cpuid",
    description: "Get CPUID information",
    usage: "cpuid",
    run: cpuid,
}];

fn cpuid(_: String) -> Result<(), String> {
    let cpuid = cpuid::cpuid();

    log::info!("{cpuid:#?}");

    Ok(())
}
