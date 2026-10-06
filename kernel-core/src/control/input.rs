use core::sync::atomic::Ordering;

use crate::sync::init::InitData;
use bitflags::bitflags;
use crossbeam_queue::ArrayQueue;
use pc_keyboard::{DecodedKey, Modifiers};
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
