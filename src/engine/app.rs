use winit::{
    application::ApplicationHandler, event::WindowEvent, keyboard::PhysicalKey, window::Window,
};

use crate::ecs::components::mesh_index::MeshIndex;
use crate::ecs::components::transform::Transform;
use crate::ecs::resources::camera::Camera;
use crate::ecs::resources::input::InputState;
use crate::ecs::resources::time::Time;
use crate::ecs::world::World;
use crate::gpu::gltf_loader;
use crate::systems::camera_system::camera_system;
use glam::Vec3;

use crate::gpu::{
    buffer::{GpuMesh, PushConstants},
    renderer::Renderer,
};

use ash::vk;

pub struct App {
    window: Option<Window>,
    renderer: Option<Renderer>,
    world: World,
    meshes: Vec<GpuMesh>,
    last_frame: std::time::Instant,
    start_time: std::time::Instant,
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            world: World::new(),
            meshes: Vec::new(),
            last_frame: std::time::Instant::now(),
            start_time: std::time::Instant::now(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes().with_title("Aelkyn"))
            .unwrap();

        window.set_cursor_visible(true);

        let renderer = Renderer::new(&window).expect("Failed to Initialize Vulkan");

        let loaded = gltf_loader::load_gltf("assets/models/firstmon.glb");
        let mesh = GpuMesh::upload(
            &renderer.device,
            &renderer.allocator,
            &renderer.commands,
            &loaded.vertices,
            &loaded.indices,
        );
        self.meshes.push(mesh);

        let player = self.world.spawn();
        self.world.insert(player, Transform::new(Vec3::ZERO));
        self.world.insert(player, MeshIndex(0));

        // Insert resources
        self.world.insert_resource(Camera::new());
        self.world.insert_resource(InputState::new());
        self.world.insert_resource(Time::new());

        self.window = Some(window);
        self.renderer = Some(renderer);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    if let Some(r) = &mut self.renderer {
                        r.resize(size.width, size.height);
                    }
                }
            }

            // Keyboard → InputState
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key_code) = event.physical_key {
                    if let Some(input) = self.world.get_resource_mut::<InputState>() {
                        if event.state.is_pressed() {
                            input.key_down(key_code);
                        } else {
                            input.key_up(key_code);
                        }
                    }
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                // We use DeviceEvent for mouse delta instead
            }

            WindowEvent::RedrawRequested => {
                let Some(renderer) = &mut self.renderer else {
                    return;
                };

                // Update time
                let now = std::time::Instant::now();
                let delta = (now - self.last_frame).as_secs_f32();
                let elapsed = (now - self.start_time).as_secs_f32();
                self.last_frame = now;

                if let Some(time) = self.world.get_resource_mut::<Time>() {
                    time.delta = delta;
                    time.elapsed = elapsed;
                }

                // Clear per-frame input
                if let Some(input) = self.world.get_resource_mut::<InputState>() {
                    input.begin_frame();
                }

                // Run systems
                camera_system(&mut self.world);

                // Get camera matrices for rendering
                let extent = renderer.swapchain.surface_resolution;
                let aspect = extent.width as f32 / extent.height.max(1) as f32;

                let vp = match self.world.get_resource::<Camera>() {
                    Some(cam) => {
                        let view = cam.view_matrix();
                        let proj = cam.projection_matrix(aspect);
                        (proj * view).to_cols_array_2d()
                    }
                    None => {
                        // Identity fallback
                        [
                            [1.0, 0.0, 0.0, 0.0],
                            [0.0, 1.0, 0.0, 0.0],
                            [0.0, 0.0, 1.0, 0.0],
                            [0.0, 0.0, 0.0, 1.0],
                        ]
                    }
                };

                // Query all renderable entities
                let renderables = self.world.query2::<Transform, MeshIndex>();

                let draw_list: Vec<_> = renderables
                    .iter()
                    .filter_map(|(_, transform, mesh_idx)| {
                        let mesh = self.meshes.get(mesh_idx.0)?;

                        let model = transform.matrix();
                        let vp_mat = glam::Mat4::from_cols_array_2d(&vp);
                        let mvp = vp_mat * model;
                        let push = PushConstants::new(mvp.to_cols_array_2d(), 0.0, elapsed);

                        let vb = mesh.vertex_buffer.buffer;
                        let ib = mesh.index_buffer.buffer;
                        let count = mesh.index_count;
                        Some((push, vb, ib, count))
                    })
                    .collect();

                renderer.draw_frame(None, |dev, cmd, layout| unsafe {
                    for (push, vb, ib, count) in &draw_list {
                        dev.cmd_push_constants(
                            cmd,
                            layout,
                            vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                            0,
                            bytemuck::bytes_of(push),
                        );
                        dev.cmd_bind_vertex_buffers(cmd, 0, &[*vb], &[0]);
                        dev.cmd_bind_index_buffer(cmd, *ib, 0, vk::IndexType::UINT32);
                        dev.cmd_draw_indexed(cmd, *count, 1, 0, 0, 0);
                    }
                });
            }
            _ => {}
        }
    }

    // Mouse delta comes through DeviceEvent for raw movement
    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let winit::event::DeviceEvent::MouseMotion { delta } = event {
            if let Some(input) = self.world.get_resource_mut::<InputState>() {
                input.mouse_delta.0 += delta.0 as f32;
                input.mouse_delta.1 += delta.1 as f32;
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
