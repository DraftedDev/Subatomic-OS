// TODO: remove '--features qemu-exit' from just-file
use crate::api;

const EXIT_PORT: u16 = 0xf4;

/// Exit codes for `Qemu` kernel tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ExitCode {
    /// Successful exit.
    Success = 0x10,
    /// Failure exit.
    Failure = 0x11,
}

/// Exits the kernel by using sending a port write to the exit device of `Qemu`.
pub fn exit(exit_code: ExitCode) {
    unsafe {
        api::port().write_u32(EXIT_PORT, exit_code as u32);
    }
}
