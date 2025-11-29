#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct TessParams {
    /// Maximum allowed angle deviation in radians.
    pub max_angle: f32,
    /// Maximum allowed geometric error in world units.
    pub max_error: f32,
}

impl Default for TessParams {
    fn default() -> Self {
        Self {
            max_angle: 0.05,
            max_error: 0.001,
        }
    }
}
