#![feature(abi_x86_interrupt)]
#![no_std]
#![allow(static_mut_refs)]
#![allow(clippy::new_without_default)]

extern crate alloc;

use kernel_core::api;
use kernel_core::api::{KernelApi, MemoryApi, PortApi, TimeApi};
use kernel_core::info::KernelApiInfo;

pub mod acpi;
pub mod cpuid;
pub mod gdt;
pub mod init;
pub mod interrupts;
pub mod memory;
pub mod port;
pub mod time;

pub const KERNEL_API: KernelApi = KernelApi {
    info: KernelApiInfo {
        package: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
    },
    init: init::init,
    setup: init::setup,
    halt: x86_64::instructions::hlt,
    disable_interrupts: x86_64::instructions::interrupts::disable,
    enable_interrupts: x86_64::instructions::interrupts::enable,
    seed: |quality| if quality { seed_quality() } else { seed_fast() },
    port: PortApi {
        read_u8: port::read_u8,
        write_u8: port::write_u8,
        read_u16: port::read_u16,
        write_u16: port::write_u16,
        read_u32: port::read_u32,
        write_u32: port::write_u32,
    },
    memory: MemoryApi {
        is_init: memory::allocator::is_init,
        alloc: memory::allocator::alloc,
        alloc_zeroed: memory::allocator::alloc_zeroed,
        dealloc: memory::allocator::dealloc,
        realloc: memory::allocator::realloc,
    },
    time: TimeApi {
        read_local: time::read_local,
        read_utc: time::read_utc,
        set_offset: time::set_offset,
    },
};

fn seed_quality() -> u64 {
    let tsc = unsafe { core::arch::x86_64::_rdtsc() };
    let rip = x86_64::instructions::read_rip().as_u64();

    // TODO: use nanosecond (implement first)
    let time = api::time().read_local().second() as u64;

    // Mix with SplitMix64
    let mut x = tsc ^ rip ^ time;
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
    x ^ (x >> 31)
}

fn seed_fast() -> u64 {
    let tsc = unsafe { core::arch::x86_64::_rdtsc() };
    let rip = x86_64::instructions::read_rip().as_u64();

    (tsc ^ rip).wrapping_mul(0x9E3779B97F4A7C15)
}
