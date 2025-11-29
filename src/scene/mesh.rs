use glam::{Vec2, Vec3};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Option<Vec2>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl MeshData {
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty() || self.indices.is_empty()
    }
}
