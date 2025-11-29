use std::collections::HashMap;

use glam::{Mat3, Mat4};

use crate::scene::{id::EntityId, mesh::MeshData, transform::Transform, visual::VisualFlags};

#[derive(Debug, Default)]
pub struct SceneWorld {
    pub(crate) entities: HashMap<EntityId, EntityRecord>,
    pub(crate) next_id: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct EntityRecord {
    pub visual: VisualFlags,
    pub transform: Transform,
    pub mesh: Option<MeshData>,
    pub model_matrix: Mat4,
    pub normal_matrix: Mat3,
}

impl SceneWorld {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn allocate_id(&mut self) -> EntityId {
        let id = EntityId(self.next_id);
        self.next_id += 1;
        id
    }
}
