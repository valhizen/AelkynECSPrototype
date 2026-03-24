use std::sync::{Arc, Mutex};

use gpu_allocator::{AllocatorReport, vulkan::*};

use super::device::Device;

/// Wrapper around `gpu_allocator::vulkan::Allocator` that is safe to share.
///
/// The inner allocator is held in an `Arc<Mutex<>>` so that the egui renderer
/// (which requires `Arc<Mutex<Allocator>>`) can share it.
pub struct GpuAllocator {
    inner: Arc<Mutex<Allocator>>,
}

impl GpuAllocator {
    pub fn new(device: &Device) -> Self {
        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: device.instance.clone(),
            device: device.device.clone(),
            physical_device: device.pdevice,
            debug_settings: Default::default(),
            // Must match the feature actually enabled at VkDevice creation.
            buffer_device_address: true,
            allocation_sizes: Default::default(),
        })
        .expect("Failed to create GPU memory allocator");
        Self {
            inner: Arc::new(Mutex::new(allocator)),
        }
    }

    pub fn allocate(&self, desc: &AllocationCreateDesc<'_>) -> Allocation {
        self.inner
            .lock()
            .unwrap_or_else(|e| e.into_inner()) // survive thread panics
            .allocate(desc)
            .expect("GPU memory allocation failed")
    }

    pub fn free(&self, allocation: Allocation) {
        self.inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .free(allocation)
            .expect("GPU memory free failed");
    }

    /// Returns a clone of the `Arc<Mutex<Allocator>>` for use by
    /// components that need shared ownership (e.g. `egui-ash-renderer`).
    pub fn inner_arc(&self) -> Arc<Mutex<Allocator>> {
        Arc::clone(&self.inner)
    }

    /// Returns a snapshot report of GPU memory usage across all heaps.
    pub fn report(&self) -> AllocatorReport {
        self.inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .generate_report()
    }
}
