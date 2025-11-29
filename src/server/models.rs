use glam::{Mat4, UVec2};

use crate::scene::{mesh::MeshData, tessellation::TessParams, visual::VisualFlags};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SubmitEntityRequest {
    pub shape: ShapePayload,
    #[serde(default)]
    pub tess_params: Option<TessParams>,
    pub entity_id: Option<u64>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SubmitEntityResponse {
    pub entity_id: u64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FlagRequest {
    pub entity_id: u64,
    pub value: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TransformRequest {
    pub entity_id: u64,
    pub matrix: MatrixPayload,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RenderRequest {
    pub camera: CameraPayload,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RenderResponse {
    pub frame_id: u64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct StateResponse {
    pub visual: VisualPayload,
    pub transform: MatrixPayload,
    pub has_mesh: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ScreenshotResponse {
    pub image_base64: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EmptyResponse {}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ShapePayload {
    pub mesh: MeshData,
}

impl ShapePayload {
    pub fn into_shape(self) -> MeshBackedShape {
        MeshBackedShape { mesh: self.mesh }
    }
}

#[derive(Debug)]
pub struct MeshBackedShape {
    pub mesh: MeshData,
}

impl crate::scene::shape::KernelShape for MeshBackedShape {
    fn tessellate(&self, _params: &TessParams) -> MeshData {
        // Already tessellated mesh.
        self.mesh.clone()
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct MatrixPayload(pub [[f32; 4]; 4]);

impl From<Mat4> for MatrixPayload {
    fn from(value: Mat4) -> Self {
        Self(value.to_cols_array_2d())
    }
}

impl From<MatrixPayload> for Mat4 {
    fn from(value: MatrixPayload) -> Self {
        Mat4::from_cols_array_2d(&value.0)
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct CameraPayload {
    pub view: MatrixPayload,
    pub proj: MatrixPayload,
    pub viewport: ViewportPayload,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct ViewportPayload(pub [u32; 2]);

impl From<UVec2> for ViewportPayload {
    fn from(v: UVec2) -> Self {
        Self([v.x, v.y])
    }
}

impl From<ViewportPayload> for UVec2 {
    fn from(v: ViewportPayload) -> Self {
        UVec2::new(v.0[0], v.0[1])
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct VisualPayload {
    pub visible: bool,
    pub selected: bool,
    pub highlighted: bool,
}

impl From<VisualFlags> for VisualPayload {
    fn from(flags: VisualFlags) -> Self {
        Self {
            visible: flags.contains(VisualFlags::VISIBLE),
            selected: flags.contains(VisualFlags::SELECTED),
            highlighted: flags.contains(VisualFlags::HIGHLIGHTED),
        }
    }
}
