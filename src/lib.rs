//! Rendering core public entrypoints.
//!
//! Exposes SceneContext (唯一のフロント) と HTTP コマンドサーバの構築器のみを公開する。

pub mod scene;
pub mod server;

#[cfg(test)]
mod tests;

pub use scene::{camera::CameraParams, context::SceneContext, error::SceneError, id::EntityId, mesh::MeshData, shape::KernelShape, tessellation::TessParams, transform::Transform, visual::VisualFlags};
pub use server::command_server;
