use core::cell::UnsafeCell;
use core::mem::MaybeUninit;

use crate::SYSTEM_INIT;

/// A container for any data prepared during system initialization.
///
/// The inner value can safely be mutated, but only before system initialization.
///
/// See [SYSTEM_INIT] for more.
pub struct InitData<T> {
    data: UnsafeCell<MaybeUninit<T>>,
    init: UnsafeCell<bool>,
}

impl<T> InitData<T> {
    /// Create a new uninitialized [InitData] instance.
    pub const fn uninit() -> Self {
        Self {
            data: UnsafeCell::new(MaybeUninit::uninit()),
            init: UnsafeCell::new(false),
        }
    }

    /// Initialize the [InitData].
    ///
    /// If the data is already set, it will simply be dropped and replaced.
    ///
    /// Will panic if [SYSTEM_INIT] is [true].
    #[allow(clippy::mut_from_ref)]
    pub fn init(&self, data: T) -> &mut T {
        if unsafe { SYSTEM_INIT } {
            panic!("Tried to mutate InitData after initialization!");
        }

        unsafe {
            let cell = &mut *self.data.get();

            if *self.init.get() {
                cell.assume_init_drop();
            }

            self.init.replace(true);
            cell.write(data)
        }
    }

    /// Get the inner data if it is already initialized.
    pub const fn try_get(&self) -> Option<&T> {
        let is_init = unsafe { *self.init.get() };

        if is_init {
            Some(unsafe { self.get_unsafe() })
        } else {
            None
        }
    }

    /// Get the inner data or panic if it's not initialized.
    ///
    /// For a zero-overhead unsafe version, see [InitData::get_unsafe].
    pub const fn get(&self) -> &T {
        if unsafe { !*self.init.get() } {
            panic!("Tried to get uninitialized InitData")
        }

        unsafe { self.get_unsafe() }
    }

    /// Get the inner data without any safety checks.
    ///
    /// Faster but unsafe version of [InitData::get].
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the inner data is actually initialized.
    pub const unsafe fn get_unsafe(&self) -> &T {
        unsafe { (&*self.data.get()).assume_init_ref() }
    }

    /// Get a mutable reference to the inner data.
    ///
    /// # Safety
    ///
    /// This is highly unsafe and should definitely not be used, like at all.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn get_mut(&self) -> &mut T {
        unsafe { (&mut *self.data.get()).assume_init_mut() }
    }
}

unsafe impl<T: Sync> Sync for InitData<T> {}

unsafe impl<T: Send> Send for InitData<T> {}
