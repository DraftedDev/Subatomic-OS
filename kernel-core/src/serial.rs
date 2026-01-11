use crate::api;
use core::fmt;
use core::fmt::Write;

const PORT: u16 = 0x3F8;

/// Prints a string to the serial console port.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::print(format_args!($($arg)*));
    };
}

/// Prints a string to the serial console port, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => {
        $crate::serial_print!("{}\n", format_args!($($arg)*));
    };
}

/// Internal function for printing to the serial console port.
pub fn print(args: fmt::Arguments) {
    SerialWriter
        .write_fmt(args)
        .expect("Printing to serial failed");
}

/// A writer that writes everything to the serial console port.
pub struct SerialWriter;

impl Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.as_bytes() {
            unsafe { api::port().write_u8(PORT, *byte) }
        }

        Ok(())
    }
}
