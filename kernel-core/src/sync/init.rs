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

            cell.write(data)
        }
    }

    /// Get the inner data.
    ///
    ///
    pub const fn get(&self) -> &T {
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
