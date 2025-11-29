use glam::{Mat4, UVec2};

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct CameraParams {
    pub view: Mat4,
    pub proj: Mat4,
    pub viewport: UVec2,
}

impl CameraParams {
    pub fn new(view: Mat4, proj: Mat4, viewport: UVec2) -> Self {
        Self { view, proj, viewport }
    }
}
