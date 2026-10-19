use crate::interrupts::apic;
use kernel_core::api;

use kernel_core::control::input::{INPUT, KeyLayout};
use pc_keyboard::{HandleControl, PS2Keyboard, ScancodeSet1};
use x86_64::structures::idt::InterruptStackFrame;

/// The global keyboard. It's only ever mutated during interrupts, so it's safe to be `mut`.
static mut KEYBOARD: PS2Keyboard<KeyLayout, ScancodeSet1> = PS2Keyboard::new(
    ScancodeSet1::new(),
    KeyLayout::De105Key,
    HandleControl::MapLettersToUnicode,
);

/// Sets the keyboard layout.
///
/// # Safety
///
/// The caller must ensure this is never called in parallel.
pub unsafe fn set_layout(layout: KeyLayout) {
    unsafe {
        KEYBOARD = PS2Keyboard::new(
            ScancodeSet1::new(),
            layout,
            HandleControl::MapLettersToUnicode,
        );
    }
}

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // read scancode from I/O port (0x60)
    let scancode = unsafe { api::port().read_u8(0x60) };

    // `KEYBOARD` and `PRODUCER` are only ever accessed here, so this is safe.
    unsafe {
        let input = INPUT.get_unsafe();

        if let Ok(Some(event)) = KEYBOARD.add_byte(scancode)
            && let Some(key) = KEYBOARD.process_keyevent(event)
        {
            input.push(key);
        }

        input.set_modifiers(KEYBOARD.get_modifiers());
    }

    unsafe {
        apic::end_of_interrupt();
    }
}
