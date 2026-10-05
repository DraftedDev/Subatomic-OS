use crate::sync::init::InitData;
use crossbeam_queue::ArrayQueue;
use pc_keyboard::DecodedKey;

/// The global [InputControl] instance.
pub static INPUT: InitData<InputControl> = InitData::uninit();

/// Controller for handling user input from the keyboard.
#[derive(Debug)]
pub struct InputControl {
    keys: ArrayQueue<DecodedKey>,
}

impl InputControl {
    const KEY_BUF_SIZE: usize = 64;

    /// Creates a new input control instance.
    pub fn new() -> Self {
        Self {
            keys: ArrayQueue::new(Self::KEY_BUF_SIZE),
        }
    }

    /// Returns whether the input queue is empty.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
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
