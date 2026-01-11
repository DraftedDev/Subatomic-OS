use x86_64::instructions::port::{PortReadOnly, PortWriteOnly};

/// Reads a `u8` from the given port.
///
/// # Safety
/// Operating on I/O ports can have unintended side effects.
pub unsafe fn read_u8(port: u16) -> u8 {
    unsafe { PortReadOnly::new(port).read() }
}

/// Write a `u8` to the given port.
///
/// # Safety
/// Operating on I/O ports can have unintended side effects.
pub unsafe fn write_u8(port: u16, value: u8) {
    unsafe { PortWriteOnly::new(port).write(value) }
}

/// Reads a `u16` from the given port.
///
/// # Safety
/// Operating on I/O ports can have unintended side effects.
pub unsafe fn read_u16(port: u16) -> u16 {
    unsafe { PortReadOnly::new(port).read() }
}

/// Write a `u16` to the given port.
///
/// # Safety
/// Operating on I/O ports can have unintended side effects.
pub unsafe fn write_u16(port: u16, value: u16) {
    unsafe { PortWriteOnly::new(port).write(value) }
}

/// Reads a `u32` from the given port.
///
/// # Safety
/// Operating on I/O ports can have unintended side effects.
pub unsafe fn read_u32(port: u16) -> u32 {
    unsafe { PortReadOnly::new(port).read() }
}

/// Write a `u32` to the given port.
///
/// # Safety
/// Operating on I/O ports can have unintended side effects.
pub unsafe fn write_u32(port: u16, value: u32) {
    unsafe { PortWriteOnly::new(port).write(value) }
}
