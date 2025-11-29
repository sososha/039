use glam::{Mat4, Quat, Vec3};

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Transform {
    pub matrix: Mat4,
}

impl Transform {
    pub fn identity() -> Self {
        Self {
            matrix: Mat4::IDENTITY,
        }
    }

    pub fn from_trs(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        let matrix = Mat4::from_scale_rotation_translation(scale, rotation, translation);
        Self { matrix }
    }
}
