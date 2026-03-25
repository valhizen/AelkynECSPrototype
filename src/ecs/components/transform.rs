use glam::{Mat3, Mat4, Quat, Vec3};

/// Transform component holding position, rotation, and scale.
///
/// Every object in the world has one of these.
#[derive(Clone)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    /// Builds the 4×4 model matrix (Scale → Rotate → Translate).
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    /// Builds the 3×3 normal matrix (inverse-transpose of the upper 3×3).
    ///
    /// Used by the fragment shader to transform normals correctly under
    /// non-uniform scale.
    pub fn normal_matrix(&self) -> Mat3 {
        Mat3::from_mat4(self.matrix()).inverse().transpose()
    }
}
