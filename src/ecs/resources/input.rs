use std::collections::HashSet;
use winit::keyboard::KeyCode;

pub struct InputState {
    pressed: HashSet<KeyCode>,
    pub mouse_delta: (f32, f32),
    pub scroll_delta: f32,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
            mouse_delta: (0.0, 0.0),
            scroll_delta: 0.0,
        }
    }

    pub fn key_down(&mut self, key: KeyCode) {
        self.pressed.insert(key);
    }

    pub fn key_up(&mut self, key: KeyCode) {
        self.pressed.remove(&key);
    }

    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    /// Call at start of each frame to clear per-frame deltas
    pub fn begin_frame(&mut self) {
        self.mouse_delta = (0.0, 0.0);
        self.scroll_delta = 0.0;
    }
}
