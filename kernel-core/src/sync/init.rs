use core::cell::UnsafeCell;
use core::mem::MaybeUninit;

/// A zero-overhead wrapper around a type that should only be initialized once.
///
/// This is a high-performance version of [spin::Once].
///
/// Should only be used for global one-time-init data.
pub struct InitData<T, M = ()> {
    data: UnsafeCell<MaybeUninit<T>>,
    _phantom: core::marker::PhantomData<M>,
}

impl<T, M> InitData<T, M> {
    /// Create a new uninitialized [InitData].
    pub const fn uninit() -> Self {
        Self {
            data: UnsafeCell::new(MaybeUninit::uninit()),
            _phantom: core::marker::PhantomData,
        }
    }

    /// Initialize the data.
    ///
    /// # Safety
    ///
    /// The caller must ensure that this is called before any access via [InitData::get].
    /// Should be called inside kernel setup functions.
    pub const unsafe fn init(&self, data: T) -> &mut T {
        unsafe {
            let inner = &mut *self.data.get();
            inner.write(data)
        }
    }

    /// Get the inner data.
    ///
    /// # Safety
    ///
    /// This is only safe, because callers of [InitData::init] face additional responsibilities.
    pub const fn get(&self) -> &T {
        unsafe { (&*self.data.get()).assume_init_ref() }
    }

    /// Get a mutable reference to the inner data.
    ///
    /// # Safety
    ///
    /// This is highly unsafe and should only be used in edge-cases.
    pub unsafe fn get_mut(&self) -> &mut T {
        unsafe { (&mut *self.data.get()).assume_init_mut() }
    }
}

unsafe impl<T: Sync> Sync for InitData<T, ()> {}

unsafe impl<T: Send> Send for InitData<T, ()> {}

/// Makes the [InitData] `Send`-safe.
pub struct SendMarker;

unsafe impl<T> Send for InitData<T, SendMarker> {}

/// Makes the [InitData] `Sync`-safe.
pub struct SyncMarker;

unsafe impl<T> Sync for InitData<T, SyncMarker> {}

/// Makes the [InitData] `Send`-safe and `Sync`-safe.
pub struct SendSyncMarker;

unsafe impl<T> Send for InitData<T, SendSyncMarker> {}
unsafe impl<T> Sync for InitData<T, SendSyncMarker> {}
