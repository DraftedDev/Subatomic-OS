use x86_64::instructions::port::{PortReadOnly, PortWriteOnly};

pub unsafe fn read_u8(port: u16) -> u8 {
    unsafe { PortReadOnly::new(port).read() }
}

pub unsafe fn write_u8(port: u16, value: u8) {
    unsafe { PortWriteOnly::new(port).write(value) }
}

pub unsafe fn read_u16(port: u16) -> u16 {
    unsafe { PortReadOnly::new(port).read() }
}

pub unsafe fn write_u16(port: u16, value: u16) {
    unsafe { PortWriteOnly::new(port).write(value) }
}

pub unsafe fn read_u32(port: u16) -> u32 {
    unsafe { PortReadOnly::new(port).read() }
}

pub unsafe fn write_u32(port: u16, value: u32) {
    unsafe { PortWriteOnly::new(port).write(value) }
}
