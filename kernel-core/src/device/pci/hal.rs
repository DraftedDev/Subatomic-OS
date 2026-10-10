use core::ptr::NonNull;
use virtio_drivers::{BufferDirection, Hal, PhysAddr};

use crate::api;

/// Kernel Hardware Abstraction Layer.
pub struct KernelHal;

fn convert_direction(dir: BufferDirection) -> api::DmaDirection {
    match dir {
        BufferDirection::DriverToDevice => api::DmaDirection::ToDevice,
        BufferDirection::DeviceToDriver => api::DmaDirection::FromDevice,
        BufferDirection::Both => api::DmaDirection::Bidirectional,
    }
}

unsafe impl Hal for KernelHal {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let (paddr, vaddr) = unsafe {
            api::memory()
                .dma_alloc(pages)
                .expect("HAL: Out of DMA memory")
        };

        let ptr = NonNull::new(vaddr as *mut u8).expect("HAL: Null DMA pointer");
        (paddr as u64, ptr)
    }

    unsafe fn dma_dealloc(p_addr: PhysAddr, v_addr: NonNull<u8>, pages: usize) -> i32 {
        unsafe {
            api::memory().dma_dealloc(p_addr as usize, v_addr.as_ptr() as usize, pages);
        }

        0
    }

    unsafe fn mmio_phys_to_virt(p_addr: PhysAddr, _size: usize) -> NonNull<u8> {
        let v_addr = unsafe { api::memory().map_to(p_addr as usize, true, false, true) };
        NonNull::new(v_addr as *mut u8).expect("HAL: Null MMIO pointer")
    }

    unsafe fn share(buffer: NonNull<[u8]>, direction: BufferDirection) -> PhysAddr {
        let v_addr = buffer.as_ptr() as *mut u8 as usize;
        let p_addr = unsafe { api::memory().translate(v_addr) };

        unsafe {
            api::memory().dma_sync_device(p_addr, buffer.len(), convert_direction(direction));
        }

        p_addr as PhysAddr
    }

    unsafe fn unshare(p_addr: PhysAddr, _buffer: NonNull<[u8]>, direction: BufferDirection) {
        unsafe {
            api::memory().dma_sync_cpu(p_addr as usize, 0, convert_direction(direction));
        }
    }
}
