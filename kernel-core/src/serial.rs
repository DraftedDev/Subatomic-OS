use crate::api;
use core::fmt;

const PORT: u16 = 0x3F8;

/// Prints a byte to the serial console.
pub fn print_byte(byte: impl Into<u8>) {
    unsafe { api::port().write_u8(PORT, byte.into()) }
}

/// Prints a string to the serial console.
pub fn print(s: impl AsRef<[u8]>) {
    for byte in s.as_ref() {
        print_byte(*byte);
    }
}

/// Prints a string to the serial console, followed by a newline.
pub fn println(s: impl AsRef<[u8]>) {
    print(s);
    print_byte(b'\n');
}

/// A writer that writes everything to the serial console port.
pub struct SerialWriter;

impl fmt::Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        print(s);

        Ok(())
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        print_byte(c as u8);

        Ok(())
    }
}
