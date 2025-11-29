use crate::scene::{mesh::MeshData, tessellation::TessParams};

/// Abstraction for kernel-provided shapes.
pub trait KernelShape {
    fn tessellate(&self, params: &TessParams) -> MeshData;
}
