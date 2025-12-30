use spin::MutexGuard;

/// A mutex that can be used to synchronize access to a value.
///
/// Similar to [spin::Mutex].
pub struct Mutex<T> {
    inner: spin::Mutex<T>,
}

impl<T> Mutex<T> {
    /// Creates a new mutex.
    pub const fn new(value: T) -> Self {
        Self {
            inner: spin::Mutex::new(value),
        }
    }

    /// Lock the value and set the data. Then unlock the value.
    pub fn set(&self, value: T) {
        let mut inner = self.lock();

        *inner = value;
    }

    /// Locks the value and runs the given closure on it.
    ///
    /// This is used to avoid deadlocks.
    pub fn run<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut lock = self.lock();

        f(&mut lock)
    }

    /// Lock the inner value.
    fn lock(&self) -> MutexGuard<'_, T> {
        self.inner.lock()
    }
}
