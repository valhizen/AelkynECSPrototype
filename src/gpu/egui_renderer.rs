use super::{allocator::GpuAllocator, commands::Commands, device::Device};
use ash::vk;
use egui_ash_renderer as ear;
use std::error::Error;

/// Wrapper around `egui-ash-renderer` for drawing egui inside a Vulkan
/// dynamic-rendering pass.
pub struct EguiRenderer {
    renderer: Option<ear::Renderer>,
}

impl EguiRenderer {
    /// Create the egui Vulkan renderer.
    ///
    /// Must be called **after** the Vulkan device and allocator are ready.
    pub fn new(
        device: &Device,
        allocator: &GpuAllocator,
        color_format: vk::Format,
    ) -> Result<Self, Box<dyn Error>> {
        let options = ear::Options {
            srgb_framebuffer: true,
            in_flight_frames: super::commands::MAX_FRAMES_IN_FLIGHT,
            ..Default::default()
        };

        let dynamic_rendering = ear::DynamicRendering {
            color_attachment_format: color_format,
            depth_attachment_format: None,
        };

        let renderer = ear::Renderer::with_gpu_allocator(
            allocator.inner_arc(),
            device.device.clone(),
            dynamic_rendering,
            options,
        )?;

        Ok(Self {
            renderer: Some(renderer),
        })
    }

    /// Record egui draw commands into the active command buffer.
    ///
    /// Call this **after** the 3D scene has been recorded but **before**
    /// ending the rendering pass.
    pub fn render(
        &mut self,
        device: &Device,
        commands: &Commands,
        cmd: vk::CommandBuffer,
        extent: vk::Extent2D,
        pixels_per_point: f32,
        clipped_primitives: Vec<egui::ClippedPrimitive>,
        textures_delta: egui::TexturesDelta,
    ) {
        if let Some(renderer) = &mut self.renderer {
            // Upload new/changed egui textures.
            renderer
                .set_textures(device.present_queue, commands.pool, &textures_delta.set)
                .expect("Failed to upload egui textures");

            // Record egui geometry draw calls.
            renderer
                .cmd_draw(cmd, extent, pixels_per_point, &clipped_primitives)
                .expect("Failed to record egui draw commands");

            // Free textures that egui no longer needs.
            renderer
                .free_textures(&textures_delta.free)
                .expect("Failed to free egui textures");
        }
    }

    pub fn destroy(&mut self) {
        // Explicitly drop the renderer so its Vulkan resources are freed
        // BEFORE the logical device and allocator are destroyed.
        self.renderer.take();
    }
}
