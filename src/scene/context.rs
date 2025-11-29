use glam::{Mat3, Mat4};

use crate::scene::{
    camera::CameraParams,
    error::{SceneError, SceneResult},
    id::EntityId,
    shape::KernelShape,
    tessellation::TessParams,
    transform::Transform,
    visual::{DirtyFlags, VisualFlags},
    world::{EntityRecord, SceneWorld},
};

#[derive(Debug)]
pub struct SceneContext {
    world: SceneWorld,
    dirty: DirtyFlags,
}

impl SceneContext {
    pub fn new() -> Self {
        Self {
            world: SceneWorld::new(),
            dirty: DirtyFlags::empty(),
        }
    }

    pub fn submit_shape<T: KernelShape>(&mut self, id: Option<EntityId>, shape: &T, params: &TessParams) -> SceneResult<EntityId> {
        let entity_id = id.unwrap_or_else(|| self.world.allocate_id());
        if self.world.entities.contains_key(&entity_id) {
            return Err(SceneError::InvalidState("entity already exists"));
        }

        let mesh = shape.tessellate(params);
        if mesh.is_empty() {
            return Err(SceneError::ResourceMissing("tessellation produced empty mesh"));
        }

        let record = EntityRecord {
            visual: VisualFlags::empty(),
            transform: Transform::identity(),
            mesh: Some(mesh),
            model_matrix: Mat4::IDENTITY,
            normal_matrix: Mat3::IDENTITY,
        };
        self.world.entities.insert(entity_id, record);
        self.dirty.insert(DirtyFlags::GEOMETRY | DirtyFlags::TRANSFORM | DirtyFlags::VISUAL);
        Ok(entity_id)
    }

    pub fn remove(&mut self, id: EntityId) -> SceneResult<()> {
        let removed = self.world.entities.remove(&id);
        if removed.is_none() {
            return Err(SceneError::UnknownEntity(id.0));
        }
        self.dirty.insert(DirtyFlags::GEOMETRY | DirtyFlags::TRANSFORM | DirtyFlags::VISUAL);
        Ok(())
    }

    pub fn set_visibility(&mut self, id: EntityId, visible: bool) -> SceneResult<()> {
        let record = self.world.entities.get_mut(&id).ok_or(SceneError::UnknownEntity(id.0))?;
        record.visual = record.visual.visible(visible);
        self.dirty.insert(DirtyFlags::VISUAL);
        Ok(())
    }

    pub fn set_highlight(&mut self, id: EntityId, highlighted: bool) -> SceneResult<()> {
        let record = self.world.entities.get_mut(&id).ok_or(SceneError::UnknownEntity(id.0))?;
        record.visual = record.visual.highlighted(highlighted);
        self.dirty.insert(DirtyFlags::VISUAL);
        Ok(())
    }

    pub fn set_selected(&mut self, id: EntityId, selected: bool) -> SceneResult<()> {
        let record = self.world.entities.get_mut(&id).ok_or(SceneError::UnknownEntity(id.0))?;
        record.visual = record.visual.selected(selected);
        self.dirty.insert(DirtyFlags::VISUAL);
        Ok(())
    }

    pub fn set_transform(&mut self, id: EntityId, transform: Transform) -> SceneResult<()> {
        let record = self.world.entities.get_mut(&id).ok_or(SceneError::UnknownEntity(id.0))?;
        record.transform = transform;
        // model matrix is identical to transform.matrix for now
        record.model_matrix = transform.matrix;
        record.normal_matrix = Mat3::from_mat4(transform.matrix).inverse().transpose();
        self.dirty.insert(DirtyFlags::TRANSFORM);
        Ok(())
    }

    pub fn render(&mut self, _camera: &CameraParams) -> SceneResult<()> {
        if !self.dirty.is_empty() {
            self.sync_gpu()?;
            debug_assert!(self.dirty.is_empty(), "dirty flags should be cleared before render");
        }
        // GPU rendering to be implemented; placeholder ensures the call path exists.
        Ok(())
    }

    pub fn sync_gpu(&mut self) -> SceneResult<()> {
        // TODO: Implement Geometry->Transform->Visual incremental sync.
        self.dirty = DirtyFlags::empty();
        Ok(())
    }

    pub fn get_state(&self, id: EntityId) -> SceneResult<EntityState> {
        let record = self.world.entities.get(&id).ok_or(SceneError::UnknownEntity(id.0))?;
        Ok(EntityState {
            visual: record.visual,
            transform: record.transform,
            has_mesh: record.mesh.is_some(),
        })
    }

    pub fn dirty_flags(&self) -> DirtyFlags {
        self.dirty
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntityState {
    pub visual: VisualFlags,
    pub transform: Transform,
    pub has_mesh: bool,
}

impl Default for SceneContext {
    fn default() -> Self {
        Self::new()
    }
}
