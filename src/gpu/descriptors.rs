use super::buffer::GpuBuffer;
use super::commands::MAX_FRAMES_IN_FLIGHT;
use super::device::Device;
use super::texture::GpuTexture;
use ash::vk;
use std::error::Error;

/// Descriptors tell shaders where to find data (UBOs, textures).
/// Layout = what bindings exist. Pool = memory for descriptor sets. Sets = actual bindings.
///
/// Set 0: Camera UBO (binding 0) + Light UBO (binding 1) + Texture sampler (binding 2)
pub struct Descriptors {
    pub pool: vk::DescriptorPool,
    pub scene_layout: vk::DescriptorSetLayout,
    pub scene_sets: [vk::DescriptorSet; MAX_FRAMES_IN_FLIGHT],
}

impl Descriptors {
    pub fn new(device: &Device) -> Result<Self, Box<dyn Error>> {
        unsafe {
            let bindings = [
                // Binding 0: Camera UBO
                vk::DescriptorSetLayoutBinding::default()
                    .binding(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT),
                // Binding 1: Light UBO
                vk::DescriptorSetLayoutBinding::default()
                    .binding(1)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT),
                // Binding 2: Texture sampler
                vk::DescriptorSetLayoutBinding::default()
                    .binding(2)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            ];

            let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
            let scene_layout = device
                .device
                .create_descriptor_set_layout(&layout_info, None)?;

            let pool_sizes = [
                vk::DescriptorPoolSize::default()
                    .ty(vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count(MAX_FRAMES_IN_FLIGHT as u32 * 2), // camera + light
                vk::DescriptorPoolSize::default()
                    .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .descriptor_count(MAX_FRAMES_IN_FLIGHT as u32),
            ];

            let pool_info = vk::DescriptorPoolCreateInfo::default()
                .max_sets(MAX_FRAMES_IN_FLIGHT as u32)
                .pool_sizes(&pool_sizes);

            let pool = device.device.create_descriptor_pool(&pool_info, None)?;

            let layouts = [scene_layout; MAX_FRAMES_IN_FLIGHT];
            let alloc_info = vk::DescriptorSetAllocateInfo::default()
                .descriptor_pool(pool)
                .set_layouts(&layouts);

            let sets_vec = device.device.allocate_descriptor_sets(&alloc_info)?;
            let scene_sets: [vk::DescriptorSet; MAX_FRAMES_IN_FLIGHT] =
                sets_vec.try_into().unwrap();

            Ok(Self {
                pool,
                scene_layout,
                scene_sets,
            })
        }
    }

    /// Binds camera UBOs, light UBOs, and a default texture to descriptor sets.
    pub fn write_scene_sets(
        &self,
        device: &Device,
        camera_ubos: &[GpuBuffer],
        camera_size: vk::DeviceSize,
        light_ubos: &[GpuBuffer],
        light_size: vk::DeviceSize,
        default_texture: &GpuTexture,
    ) {
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let camera_info = vk::DescriptorBufferInfo::default()
                .buffer(camera_ubos[i].buffer)
                .offset(0)
                .range(camera_size);

            let light_info = vk::DescriptorBufferInfo::default()
                .buffer(light_ubos[i].buffer)
                .offset(0)
                .range(light_size);

            let image_info = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(default_texture.view)
                .sampler(default_texture.sampler);

            let writes = [
                vk::WriteDescriptorSet::default()
                    .dst_set(self.scene_sets[i])
                    .dst_binding(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(std::slice::from_ref(&camera_info)),
                vk::WriteDescriptorSet::default()
                    .dst_set(self.scene_sets[i])
                    .dst_binding(1)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(std::slice::from_ref(&light_info)),
                vk::WriteDescriptorSet::default()
                    .dst_set(self.scene_sets[i])
                    .dst_binding(2)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(std::slice::from_ref(&image_info)),
            ];

            unsafe {
                device.device.update_descriptor_sets(&writes, &[]);
            }
        }
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.device.destroy_descriptor_pool(self.pool, None);
            device
                .device
                .destroy_descriptor_set_layout(self.scene_layout, None);
        }
    }
}
