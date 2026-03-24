use super::{allocator::GpuAllocator, commands::Commands, device::Device};
use ash::vk;
use gpu_allocator::{
    MemoryLocation,
    vulkan::{Allocation, AllocationCreateDesc, AllocationScheme},
};

/// A GPU-resident texture: VkImage + VkImageView + VkSampler + memory.
pub struct GpuTexture {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub sampler: vk::Sampler,
    allocation: Option<Allocation>,
    pub width: u32,
    pub height: u32,
}

impl GpuTexture {
    /// Load a texture from a file path (PNG, JPEG, etc.).
    pub fn from_file(
        device: &Device,
        allocator: &GpuAllocator,
        commands: &Commands,
        path: &str,
    ) -> Self {
        let img = image::open(path)
            .unwrap_or_else(|e| panic!("Failed to load texture '{path}': {e}"))
            .to_rgba8();
        let (w, h) = img.dimensions();
        Self::from_rgba(device, allocator, commands, w, h, &img)
    }

    /// Create a texture from raw RGBA8 pixel data.
    pub fn from_rgba(
        device: &Device,
        allocator: &GpuAllocator,
        commands: &Commands,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> Self {
        let format = vk::Format::R8G8B8A8_SRGB;
        let size = (width * height * 4) as vk::DeviceSize;

        // Create staging buffer with pixel data.
        let staging_buf = unsafe {
            let info = vk::BufferCreateInfo::default()
                .size(size)
                .usage(vk::BufferUsageFlags::TRANSFER_SRC)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);
            let buffer = device.device.create_buffer(&info, None).unwrap();
            let requirements = device.device.get_buffer_memory_requirements(buffer);

            let alloc = allocator.allocate(&AllocationCreateDesc {
                name: "texture_staging",
                requirements,
                location: MemoryLocation::CpuToGpu,
                linear: true,
                allocation_scheme: AllocationScheme::GpuAllocatorManaged,
            });

            device
                .device
                .bind_buffer_memory(buffer, alloc.memory(), alloc.offset())
                .unwrap();

            // Copy pixel data into staging buffer.
            let ptr = alloc.mapped_ptr().expect("Staging buffer not mapped");
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr.as_ptr() as *mut u8, data.len());

            (buffer, alloc)
        };

        // Create the GPU image.
        let (image, alloc) = unsafe {
            let info = vk::ImageCreateInfo::default()
                .image_type(vk::ImageType::TYPE_2D)
                .format(format)
                .extent(vk::Extent3D {
                    width,
                    height,
                    depth: 1,
                })
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let image = device.device.create_image(&info, None).unwrap();
            let requirements = device.device.get_image_memory_requirements(image);

            let alloc = allocator.allocate(&AllocationCreateDesc {
                name: "texture_image",
                requirements,
                location: MemoryLocation::GpuOnly,
                linear: false,
                allocation_scheme: AllocationScheme::DedicatedImage(image),
            });

            device
                .device
                .bind_image_memory(image, alloc.memory(), alloc.offset())
                .unwrap();

            (image, alloc)
        };

        // Transition → TRANSFER_DST, copy staging → image, transition → SHADER_READ.
        commands.run_one_time(device, |dev, cmd| unsafe {
            // Undefined → TransferDst
            let to_transfer = vk::ImageMemoryBarrier::default()
                .image(image)
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_access_mask(vk::AccessFlags::NONE)
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .subresource_range(Self::color_subresource());
            dev.cmd_pipeline_barrier(
                cmd,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[to_transfer],
            );

            // Copy buffer → image.
            let region = vk::BufferImageCopy::default()
                .buffer_offset(0)
                .buffer_row_length(0)
                .buffer_image_height(0)
                .image_subresource(vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: 0,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                .image_extent(vk::Extent3D {
                    width,
                    height,
                    depth: 1,
                });
            dev.cmd_copy_buffer_to_image(
                cmd,
                staging_buf.0,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );

            // TransferDst → ShaderReadOnly
            let to_shader = vk::ImageMemoryBarrier::default()
                .image(image)
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ)
                .subresource_range(Self::color_subresource());
            dev.cmd_pipeline_barrier(
                cmd,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[to_shader],
            );
        });

        // Clean up staging buffer.
        unsafe {
            device.device.destroy_buffer(staging_buf.0, None);
        }
        allocator.free(staging_buf.1);

        // Create image view.
        let view = unsafe {
            let info = vk::ImageViewCreateInfo::default()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format)
                .subresource_range(Self::color_subresource());
            device.device.create_image_view(&info, None).unwrap()
        };

        // Create sampler.
        let sampler = unsafe {
            let info = vk::SamplerCreateInfo::default()
                .mag_filter(vk::Filter::LINEAR)
                .min_filter(vk::Filter::LINEAR)
                .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
                .address_mode_u(vk::SamplerAddressMode::REPEAT)
                .address_mode_v(vk::SamplerAddressMode::REPEAT)
                .address_mode_w(vk::SamplerAddressMode::REPEAT)
                .anisotropy_enable(true)
                .max_anisotropy(16.0)
                .min_lod(0.0)
                .max_lod(1.0);
            device.device.create_sampler(&info, None).unwrap()
        };

        Self {
            image,
            view,
            sampler,
            allocation: Some(alloc),
            width,
            height,
        }
    }

    /// Creates a 1×1 white texture for objects without a texture map.
    pub fn default_white(device: &Device, allocator: &GpuAllocator, commands: &Commands) -> Self {
        Self::from_rgba(device, allocator, commands, 1, 1, &[255, 255, 255, 255])
    }

    pub fn destroy(&mut self, device: &Device, allocator: &GpuAllocator) {
        unsafe {
            device.device.destroy_sampler(self.sampler, None);
            device.device.destroy_image_view(self.view, None);
            device.device.destroy_image(self.image, None);
        }
        if let Some(alloc) = self.allocation.take() {
            allocator.free(alloc);
        }
    }

    fn color_subresource() -> vk::ImageSubresourceRange {
        vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        }
    }
}
