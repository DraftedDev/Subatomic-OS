//! Core library for Subatomic OS.

#![no_std]
#![allow(static_mut_refs)]
#![warn(missing_docs)]

extern crate alloc;

/// Contains the [api::KernelApi] and related types.
pub mod api;

/// Contains the [info::KernelInfo] and related types.
pub mod info;

/// Contains the global logger infrastructure.
pub mod logger;

/// Contains limine request globals.
pub mod requests;

/// Contains serial port infrastructure.
pub mod serial;

/// Contains synchronization primitives.
pub mod sync;

/// Contains support for interacting with `Qemu`.
pub mod qemu;

/// Contains the control infrastructure.
pub mod control;

/// Contains the [wrapper::UnsafeWrapper] to unsafely implement traits like [Send] and [Sync].
pub mod wrapper;

/// Contains styling functionality using [ustyle].
pub mod style;

/// Contains types for organized data collections.
pub mod collections;

/// Contains time handling using the [time] crate.
pub mod time;
