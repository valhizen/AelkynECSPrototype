use glam::Mat4;
use glam::Vec3;

pub enum CameraMode {
    ThirdPerson,
    FirstPerson,
    Debug,
}

pub struct Camera {
    pub position: glam::Vec3,
    pub front: glam::Vec3,
    pub up: glam::Vec3,
    pub right: glam::Vec3,
    pub world_up: glam::Vec3,

    pub yaw: f32,
    pub pitch: f32,

    pub movement_speed: f32,
    pub mouse_sensitivity: f32,
    pub zoom: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(0.0, 1.5, 5.0),
            front: Vec3::new(0.0, 0.0, -1.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            right: Vec3::new(1.0, 0.0, 0.0),
            world_up: Vec3::new(0.0, 1.0, 0.0),
            yaw: -90.0_f32,
            pitch: 0.0,
            movement_speed: 5.0,
            mouse_sensitivity: 0.1,
            zoom: 45.0,
        }
    }

    pub fn update_vectors(&mut self) {
        let front = Vec3::new(
            self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        );
        self.front = front.normalize();
        self.right = self.front.cross(self.world_up).normalize();
        self.up = self.right.cross(self.front).normalize();
    }
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.position + self.front, self.up)
    }

    pub fn projection_matrix(&self, aspect: f32) -> Mat4 {
        let mut proj = Mat4::perspective_rh(self.zoom.to_radians(), aspect, 0.1, 100.0);
        proj.y_axis.y *= -1.0;
        proj
    }
}
