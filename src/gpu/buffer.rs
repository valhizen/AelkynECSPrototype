use super::{allocator::GpuAllocator, commands::Commands, device::Device};
use ash::vk;
use gpu_allocator::{
    MemoryLocation,
    vulkan::{Allocation, AllocationCreateDesc, AllocationScheme},
};

// ---------------------------------------------------------------------------
// Vertex
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex {
    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(std::mem::size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
    }

    pub fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 4] {
        [
            // location 0: position
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(0),
            // location 1: normal
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(12),
            // location 2: color
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(24),
            // location 3: uv
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(3)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(36),
        ]
    }
}

// ---------------------------------------------------------------------------
// Push Constants — small per-object data sent inline with draw commands (no buffer needed).
// Faster than UBOs for data that changes every draw call (like model matrix).
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PushConstants {
    pub model: [[f32; 4]; 4],
    pub tex_blend: f32,
    pub time: f32,
    _pad: [f32; 2],
}

impl PushConstants {
    pub fn new(model: [[f32; 4]; 4], tex_blend: f32, time: f32) -> Self {
        Self {
            model,
            tex_blend,
            time,
            _pad: [0.0; 2],
        }
    }
}

// ---------------------------------------------------------------------------
// GpuBuffer — wraps a VkBuffer + its memory allocation. The building block for
// all GPU data: vertex buffers, index buffers, uniform buffers.
// ---------------------------------------------------------------------------

pub struct GpuBuffer {
    pub buffer: vk::Buffer,
    pub allocation: Option<Allocation>,
    pub size: vk::DeviceSize,
}

impl GpuBuffer {
    pub fn new(
        device: &Device,
        allocator: &GpuAllocator,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        location: MemoryLocation,
        name: &str,
    ) -> Self {
        unsafe {
            let info = vk::BufferCreateInfo::default()
                .size(size)
                .usage(usage)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let buffer = device.device.create_buffer(&info, None).unwrap();
            let requirements = device.device.get_buffer_memory_requirements(buffer);

            let allocation = allocator.allocate(&AllocationCreateDesc {
                name,
                requirements,
                location,
                linear: true,
                allocation_scheme: AllocationScheme::GpuAllocatorManaged,
            });

            device
                .device
                .bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
                .unwrap();

            Self {
                buffer,
                allocation: Some(allocation),
                size,
            }
        }
    }

    pub fn destroy(&mut self, device: &Device, allocator: &GpuAllocator) {
        if let Some(alloc) = self.allocation.take() {
            allocator.free(alloc);
        }
        unsafe {
            device.device.destroy_buffer(self.buffer, None);
        }
    }

    pub fn write<T: bytemuck::NoUninit>(&self, data: &T) {
        let alloc = self.allocation.as_ref().expect("Buffer already destroyed");
        let ptr = alloc
            .mapped_ptr()
            .expect("Buffer not mapped — was it created with CpuToGpu?");
        unsafe {
            std::ptr::copy_nonoverlapping(
                bytemuck::bytes_of(data).as_ptr(),
                ptr.as_ptr() as *mut u8,
                std::mem::size_of::<T>(),
            );
        }
    }
}

// ---------------------------------------------------------------------------
// GpuMesh  — vertex + index buffers uploaded once via a staging buffer
// ---------------------------------------------------------------------------

pub struct GpuMesh {
    pub vertex_buffer: GpuBuffer,
    pub index_buffer: GpuBuffer,
    pub index_count: u32,
}

impl GpuMesh {
    /// Uploads vertices and indices to GPU-only memory using a temporary staging buffer.
    pub fn upload(
        device: &Device,
        allocator: &GpuAllocator,
        commands: &Commands,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> Self {
        let vb = Self::upload_buffer(
            device,
            allocator,
            commands,
            bytemuck::cast_slice(vertices),
            vk::BufferUsageFlags::VERTEX_BUFFER,
            "vertex_buffer",
        );

        let ib = Self::upload_buffer(
            device,
            allocator,
            commands,
            bytemuck::cast_slice(indices),
            vk::BufferUsageFlags::INDEX_BUFFER,
            "index_buffer",
        );

        Self {
            vertex_buffer: vb,
            index_buffer: ib,
            index_count: indices.len() as u32,
        }
    }

    /// Copies `data` into a GPU-only buffer via a short-lived staging buffer.
    fn upload_buffer(
        device: &Device,
        allocator: &GpuAllocator,
        commands: &Commands,
        data: &[u8],
        usage: vk::BufferUsageFlags,
        name: &str,
    ) -> GpuBuffer {
        let size = data.len() as vk::DeviceSize;

        // Staging buffer — CPU visible, used only for the upload.
        let mut staging = GpuBuffer::new(
            device,
            allocator,
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryLocation::CpuToGpu,
            &format!("{name}_staging"),
        );
        // Write the data into the staging buffer directly.
        let alloc = staging.allocation.as_ref().unwrap();
        let ptr = alloc.mapped_ptr().expect("Staging buffer not mapped");
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr.as_ptr() as *mut u8, data.len());
        }

        // GPU-only destination buffer.
        let dst = GpuBuffer::new(
            device,
            allocator,
            size,
            usage | vk::BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuOnly,
            name,
        );

        // Record and immediately submit a one-shot copy command.
        commands.run_one_time(device, |dev, cmd| unsafe {
            dev.cmd_copy_buffer(
                cmd,
                staging.buffer,
                dst.buffer,
                &[vk::BufferCopy {
                    src_offset: 0,
                    dst_offset: 0,
                    size,
                }],
            );
        });

        // Staging buffer is no longer needed.
        staging.destroy(device, allocator);

        dst
    }

    /// Frees both GPU buffers.
    pub fn destroy(mut self, device: &Device, allocator: &GpuAllocator) {
        self.vertex_buffer.destroy(device, allocator);
        self.index_buffer.destroy(device, allocator);
    }
}
