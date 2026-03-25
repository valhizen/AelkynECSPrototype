pub struct Time {
    pub delta: f32,
    pub elapsed: f32,
}

impl Time {
    pub fn new() -> Self {
        Self {
            delta: 0.0,
            elapsed: 0.0,
        }
    }
}
