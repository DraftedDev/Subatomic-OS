use core::{str::FromStr, sync::atomic::Ordering};

use crate::sync::init::InitData;
use bitflags::bitflags;
use crossbeam_queue::ArrayQueue;
use pc_keyboard::{
    DecodedKey, HandleControl, KeyCode, KeyboardLayout, Modifiers, PhysicalKeyboard,
    layouts::{Azerty, Colemak, DVP104Key, De105Key, Dvorak104Key, Jis109Key, Uk105Key, Us104Key},
};
use portable_atomic::AtomicU8;

/// The global [InputControl] instance.
pub static INPUT: InitData<InputControl> = InitData::uninit();

/// Controller for handling user input from the keyboard.
#[derive(Debug)]
pub struct InputControl {
    keys: ArrayQueue<DecodedKey>,
    mods: AtomicU8,
}

impl InputControl {
    const KEY_BUF_SIZE: usize = 64;

    /// Creates a new input control instance.
    pub fn new() -> Self {
        Self {
            keys: ArrayQueue::new(Self::KEY_BUF_SIZE),
            mods: AtomicU8::new(0),
        }
    }

    /// Returns whether the input queue is empty.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Sets the current modifier flags based on the given modifiers.
    pub fn set_modifiers(&self, mods: &Modifiers) {
        let mut flags = ModifierFlags::empty();

        if mods.rshift {
            flags.insert(ModifierFlags::R_SHIFT);
        }

        if mods.rshift {
            flags.insert(ModifierFlags::R_SHIFT);
        }

        if mods.lctrl {
            flags.insert(ModifierFlags::L_CTRL);
        }

        if mods.rctrl {
            flags.insert(ModifierFlags::R_CTRL);
        }

        if mods.numlock {
            flags.insert(ModifierFlags::NUMLOCK);
        }

        if mods.capslock {
            flags.insert(ModifierFlags::CAPSLOCK);
        }

        if mods.lalt {
            flags.insert(ModifierFlags::L_ALT);
        }

        if mods.ralt {
            flags.insert(ModifierFlags::R_ALT);
        }

        self.mods.store(flags.bits(), Ordering::Relaxed);
    }

    /// Returns the current modifier flags.
    pub fn modifiers(&self) -> ModifierFlags {
        ModifierFlags::from_bits_retain(self.mods.load(Ordering::Relaxed))
    }

    /// Push a key to the queue.
    pub fn push(&self, key: DecodedKey) {
        let _ = self.keys.push(key);
    }

    /// Pop the next key from the queue.
    pub fn pop(&self) -> Option<DecodedKey> {
        self.keys.pop()
    }
}

bitflags! {
    /// Represents the current keyboard modifier flags.
    pub struct ModifierFlags: u8 {
        /// Left shift key is pressed.
        const L_SHIFT = 1 << 0;
        /// Right shift key is pressed.
        const R_SHIFT = 1 << 1;
        /// Left control key is pressed.
        const L_CTRL = 1 << 2;
        /// Right control key is pressed.
        const R_CTRL = 1 << 3;
        /// Numlock key is pressed.
        const NUMLOCK = 1 << 4;
        /// Capslock key is pressed.
        const CAPSLOCK = 1 << 5;
        /// Left alt key is pressed.
        const L_ALT = 1 << 6;
        /// Right alt key is pressed.
        const R_ALT = 1 << 7;
    }
}

/// The layout of a PS/2 or USB keyboard.
#[derive(Copy, Clone, Debug)]
pub enum KeyLayout {
    /// See [DVP104Key].
    DVP104Key,
    /// See [Dvorak104Key].
    Dvorak104Key,
    /// See [Us104Key].
    Us104Key,
    /// See [Uk105Key].
    Uk105Key,
    /// See [Jis109Key].
    Jis109Key,
    /// See [Azerty].
    Azerty,
    /// See [Colemak].
    Colemak,
    /// See [De105Key].
    De105Key,
}

impl KeyLayout {
    /// Get the keyboard layout as string.
    pub const fn as_str(&self) -> &str {
        match self {
            KeyLayout::DVP104Key => "dvp104",
            KeyLayout::Dvorak104Key => "dvorak104",
            KeyLayout::Us104Key => "us104",
            KeyLayout::Uk105Key => "uk105",
            KeyLayout::Jis109Key => "jis109",
            KeyLayout::Azerty => "azerty",
            KeyLayout::Colemak => "colemak",
            KeyLayout::De105Key => "de105",
        }
    }
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

    fn get_physical(&self) -> PhysicalKeyboard {
        match self {
            KeyLayout::DVP104Key => PhysicalKeyboard::Ansi,
            KeyLayout::Dvorak104Key => PhysicalKeyboard::Ansi,
            KeyLayout::Us104Key => PhysicalKeyboard::Ansi,
            KeyLayout::Uk105Key => PhysicalKeyboard::Iso,
            KeyLayout::Jis109Key => PhysicalKeyboard::Jis,
            KeyLayout::Azerty => PhysicalKeyboard::Iso,
            KeyLayout::Colemak => PhysicalKeyboard::Ansi,
            KeyLayout::De105Key => PhysicalKeyboard::Iso,
        }
    }
}

impl FromStr for KeyLayout {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dvp104" => Ok(Self::DVP104Key),
            "dvorak104" => Ok(Self::Dvorak104Key),
            "us104" => Ok(Self::Us104Key),
            "uk105" => Ok(Self::Uk105Key),
            "jis109" => Ok(Self::Jis109Key),
            "azerty" => Ok(Self::Azerty),
            "colemak" => Ok(Self::Colemak),
            "de105" => Ok(Self::De105Key),
            _ => Err(()),
        }
    }
}
