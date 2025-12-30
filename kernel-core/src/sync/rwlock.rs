use spin::{RwLockReadGuard, RwLockWriteGuard};

/// A read-write-lock that can be used to synchronize access to a value.
///
/// This is a safer version of [spin::RwLock].
pub struct RwLock<T> {
    inner: spin::RwLock<T>,
}

impl<T> RwLock<T> {
    /// Creates a new read-write-lock.
    pub const fn new(value: T) -> Self {
        Self {
            inner: spin::RwLock::new(value),
        }
    }

    /// Lock the value and set the data. Then unlock the value.
    pub fn set(&self, value: T) {
        let mut inner = self.write();

        *inner = value;
    }

    /// Locks the value and runs the given closure on it.
    ///
    /// This is used to avoid deadlocks.
    pub fn run<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let lock = self.read();

        f(&lock)
    }

    /// Locks the value and runs the given closure on it.
    ///
    /// This is used to avoid deadlocks.
    pub fn run_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut lock = self.write();

        f(&mut lock)
    }

    /// Acquire read-lock.
    fn read(&self) -> RwLockReadGuard<'_, T> {
        self.inner.read()
    }

    /// Acquire write-lock.
    fn write(&self) -> RwLockWriteGuard<'_, T> {
        self.inner.write()
    }
}
