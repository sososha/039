use super::{context::SceneContext, error::SceneError, id::EntityId, shape::KernelShape, tessellation::TessParams, visual::DirtyFlags};
use crate::scene::mesh::{MeshData, Vertex};

struct DummyShape {
    mesh: MeshData,
}

impl KernelShape for DummyShape {
    fn tessellate(&self, _params: &TessParams) -> MeshData {
        self.mesh.clone()
    }
}

fn simple_triangle() -> MeshData {
    MeshData {
        vertices: vec![
            Vertex { position: (0.0, 0.0, 0.0).into(), normal: (0.0, 0.0, 1.0).into(), uv: None },
            Vertex { position: (1.0, 0.0, 0.0).into(), normal: (0.0, 0.0, 1.0).into(), uv: None },
            Vertex { position: (0.0, 1.0, 0.0).into(), normal: (0.0, 0.0, 1.0).into(), uv: None },
        ],
        indices: vec![0, 1, 2],
    }
}

#[test]
fn submit_sets_dirty_and_returns_id() {
    let mut ctx = SceneContext::new();
    let shape = DummyShape { mesh: simple_triangle() };
    let id = ctx.submit_shape(None, &shape, &TessParams::default()).unwrap();
    assert_eq!(id.0, 1);
    assert!(ctx.dirty_flags().contains(DirtyFlags::GEOMETRY | DirtyFlags::TRANSFORM | DirtyFlags::VISUAL));
}

#[test]
fn remove_unknown_entity_errors() {
    let mut ctx = SceneContext::new();
    let err = ctx.remove(EntityId(999)).unwrap_err();
    matches!(err, SceneError::UnknownEntity(999));
}

#[test]
fn empty_mesh_rejected() {
    let mut ctx = SceneContext::new();
    let empty_shape = DummyShape { mesh: MeshData { vertices: vec![], indices: vec![] } };
    let err = ctx.submit_shape(None, &empty_shape, &TessParams::default()).unwrap_err();
    matches!(err, SceneError::ResourceMissing(_));
}

#[test]
fn sync_clears_dirty_before_render() {
    let mut ctx = SceneContext::new();
    let shape = DummyShape { mesh: simple_triangle() };
    let _ = ctx.submit_shape(None, &shape, &TessParams::default()).unwrap();
    let camera = crate::scene::camera::CameraParams::new(glam::Mat4::IDENTITY, glam::Mat4::IDENTITY, glam::UVec2::new(800, 600));
    ctx.render(&camera).unwrap();
    assert!(ctx.dirty_flags().is_empty());
}
