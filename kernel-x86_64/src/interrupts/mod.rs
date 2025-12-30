pub mod apic;
pub mod exceptions;
pub mod idt;
pub mod keyboard;
pub mod timer;

pub const INTERRUPT_OFFSET: u8 = 32;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptVector {
    Timer = 0,
    Keyboard = 1,
    Error = 19,
    Spurious = 31,
}

impl InterruptVector {
    pub fn with_offset(self) -> u8 {
        self as u8 + INTERRUPT_OFFSET
    }
}
