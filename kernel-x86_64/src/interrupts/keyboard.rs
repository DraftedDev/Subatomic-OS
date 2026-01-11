use crate::interrupts::apic;
use kernel_core::api;

use kernel_core::control::input::INPUT;
use pc_keyboard::layouts::{
    Azerty, Colemak, DVP104Key, De105Key, Dvorak104Key, Jis109Key, Uk105Key, Us104Key,
};
use pc_keyboard::{
    DecodedKey, HandleControl, KeyCode, Keyboard, KeyboardLayout, Modifiers, ScancodeSet1,
};
use x86_64::structures::idt::InterruptStackFrame;

/// The global keyboard. It's only ever mutated during interrupts, so it's safe to be `mut`.
static mut KEYBOARD: Keyboard<KeyLayout, ScancodeSet1> = Keyboard::new(
    ScancodeSet1::new(),
    KeyLayout::De105Key,
    HandleControl::MapLettersToUnicode,
);

/// Sets the keyboard layout.
pub fn set_layout(layout: KeyLayout) {
    // This is safe, because it's never called inside the interrupt handler.
    unsafe {
        KEYBOARD = Keyboard::new(
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
        if let Ok(Some(event)) = KEYBOARD.add_byte(scancode)
            && let Some(key) = KEYBOARD.process_keyevent(event)
        {
            INPUT.get().push(key);
        }
    }

    unsafe {
        apic::end_of_interrupt();
    }
}

pub enum KeyLayout {
    DVP104Key,
    Dvorak104Key,
    Us104Key,
    Uk105Key,
    Jis109Key,
    Azerty,
    Colemak,
    De105Key,
}

impl KeyboardLayout for KeyLayout {
    fn map_keycode(
        &self,
        keycode: KeyCode,
        modifiers: &Modifiers,
        handle_ctrl: HandleControl,
    ) -> DecodedKey {
        match self {
            KeyLayout::DVP104Key => DVP104Key.map_keycode(keycode, modifiers, handle_ctrl),
            KeyLayout::Dvorak104Key => Dvorak104Key.map_keycode(keycode, modifiers, handle_ctrl),
            KeyLayout::Us104Key => Us104Key.map_keycode(keycode, modifiers, handle_ctrl),
            KeyLayout::Uk105Key => Uk105Key.map_keycode(keycode, modifiers, handle_ctrl),
            KeyLayout::Jis109Key => Jis109Key.map_keycode(keycode, modifiers, handle_ctrl),
            KeyLayout::Azerty => Azerty.map_keycode(keycode, modifiers, handle_ctrl),
            KeyLayout::Colemak => Colemak.map_keycode(keycode, modifiers, handle_ctrl),
            KeyLayout::De105Key => De105Key.map_keycode(keycode, modifiers, handle_ctrl),
        }
    }
}
