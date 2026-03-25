use crate::ecs::resources::camera::Camera;
use crate::ecs::resources::input::InputState;
use crate::ecs::resources::time::Time;
use crate::ecs::world::World;
use winit::keyboard::KeyCode;

pub fn camera_system(world: &mut World) {
    // Read time
    let dt = match world.get_resource::<Time>() {
        Some(t) => t.delta,
        None => return,
    };

    // Read input state — collect what we need as plain values
    let (keys_w, keys_s, keys_a, keys_d, keys_space, keys_ctrl, mouse_dx, mouse_dy, scroll) =
        match world.get_resource::<InputState>() {
            Some(input) => (
                input.is_pressed(KeyCode::KeyW),
                input.is_pressed(KeyCode::KeyS),
                input.is_pressed(KeyCode::KeyA),
                input.is_pressed(KeyCode::KeyD),
                input.is_pressed(KeyCode::Space),
                input.is_pressed(KeyCode::ControlLeft),
                input.mouse_delta.0,
                input.mouse_delta.1,
                input.scroll_delta,
            ),
            None => return,
        };

    // Now get camera mutably — no other borrows active
    let camera = match world.get_resource_mut::<Camera>() {
        Some(c) => c,
        None => return,
    };

    // --- Keyboard movement (processKeyboard) ---
    let velocity = camera.movement_speed * dt;

    if keys_w {
        camera.position += camera.front * velocity;
    }
    if keys_s {
        camera.position -= camera.front * velocity;
    }
    if keys_a {
        camera.position -= camera.right * velocity;
    }
    if keys_d {
        camera.position += camera.right * velocity;
    }
    if keys_space {
        camera.position += camera.up * velocity;
    }
    if keys_ctrl {
        camera.position -= camera.up * velocity;
    }

    // --- Mouse rotation (processMouseMovement) ---
    camera.yaw += mouse_dx * camera.mouse_sensitivity;
    camera.pitch += mouse_dy * camera.mouse_sensitivity;

    // Clamp pitch so camera doesn't flip
    if camera.pitch > 89.0 {
        camera.pitch = 89.0;
    }
    if camera.pitch < -89.0 {
        camera.pitch = -89.0;
    }

    // --- Scroll zoom (processMouseScroll) ---
    camera.zoom -= scroll;
    if camera.zoom < 1.0 {
        camera.zoom = 1.0;
    }
    if camera.zoom > 45.0 {
        camera.zoom = 45.0;
    }

    // --- Recalculate vectors (updateCameraVectors) ---
    camera.update_vectors();
}
