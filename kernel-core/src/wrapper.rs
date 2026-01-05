use core::ops::{Deref, DerefMut};

/// A wrapper that implements [Send].
///
/// See [UnsafeWrapper] for more.
pub type SendWrapper<T> = UnsafeWrapper<T, ImplSend, ()>;

/// A wrapper that implements [Sync].
///
/// See [UnsafeWrapper] for more.
pub type SyncWrapper<T> = UnsafeWrapper<T, (), ImplSync>;

/// A wrapper that implements both [Send] and [Sync].
///
/// See [UnsafeWrapper] for more.
pub type SendSyncWrapper<T> = UnsafeWrapper<T, ImplSend, ImplSync>;

/// A highly unsafe wrapper to unsafely implement traits like [Send] or [Sync].
///
/// This is highly unsafe and should only be used with caution,
/// as it bypasses Rust's trait bounds.
pub struct UnsafeWrapper<T, SEND, SYNC> {
    inner: T,
    _data: core::marker::PhantomData<(SEND, SYNC)>,
}

impl<T, SEND, SYNC> UnsafeWrapper<T, SEND, SYNC> {
    /// Creates a new unsafe wrapper.
    ///
    /// # Safety
    /// This function is only unsafe, because the [UnsafeWrapper] itself is unsafe.
    /// It does not actually do anything unsafe.
    pub unsafe fn new(inner: T) -> Self {
        Self {
            inner,
            _data: core::marker::PhantomData,
        }
    }
}

impl<T, SEND, SYNC> Deref for UnsafeWrapper<T, SEND, SYNC> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, SEND, SYNC> DerefMut for UnsafeWrapper<T, SEND, SYNC> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// A marker struct for [UnsafeWrapper] that implements [Send].
pub struct ImplSend;

unsafe impl<T, SYNC> Send for UnsafeWrapper<T, ImplSend, SYNC> {}

/// A marker struct for [UnsafeWrapper] that implements [Sync].
pub struct ImplSync;

unsafe impl<T, SEND> Sync for UnsafeWrapper<T, SEND, ImplSync> {}
