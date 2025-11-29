pub mod models;

use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, routing::{delete, get, post}, Json, Router};
use tokio::sync::Mutex;

use crate::scene::{camera::CameraParams, context::SceneContext, error::SceneError, id::EntityId, transform::Transform};

use self::models::*;

pub type SharedContext = Arc<Mutex<SceneContext>>;

pub fn command_server(ctx: SharedContext) -> Router {
    Router::new()
        .route("/api/entity", post(submit_entity))
        .route("/api/entity/:id", delete(remove_entity))
        .route("/api/select", post(select))
        .route("/api/highlight", post(highlight))
        .route("/api/visibility", post(visibility))
        .route("/api/transform", post(transform))
        .route("/api/render", post(render))
        .route("/api/state/:id", get(get_state))
        .route("/api/screenshot", get(screenshot))
        .with_state(ctx)
}

async fn submit_entity(State(ctx): State<SharedContext>, Json(req): Json<SubmitEntityRequest>) -> Result<Json<SubmitEntityResponse>, ApiError> {
    let mut ctx = ctx.lock().await;
    let shape = req.shape.into_shape();
    let id = ctx
        .submit_shape(req.entity_id.map(EntityId), &shape, &req.tess_params.unwrap_or_default())
        .map_err(ApiError::from)?;
    Ok(Json(SubmitEntityResponse { entity_id: id.0 }))
}

async fn remove_entity(State(ctx): State<SharedContext>, axum::extract::Path(id): axum::extract::Path<u64>) -> Result<Json<EmptyResponse>, ApiError> {
    let mut ctx = ctx.lock().await;
    ctx.remove(EntityId(id)).map_err(ApiError::from)?;
    Ok(Json(EmptyResponse {}))
}

async fn select(State(ctx): State<SharedContext>, Json(req): Json<FlagRequest>) -> Result<Json<EmptyResponse>, ApiError> {
    let mut ctx = ctx.lock().await;
    ctx.set_selected(EntityId(req.entity_id), req.value).map_err(ApiError::from)?;
    Ok(Json(EmptyResponse {}))
}

async fn highlight(State(ctx): State<SharedContext>, Json(req): Json<FlagRequest>) -> Result<Json<EmptyResponse>, ApiError> {
    let mut ctx = ctx.lock().await;
    ctx.set_highlight(EntityId(req.entity_id), req.value).map_err(ApiError::from)?;
    Ok(Json(EmptyResponse {}))
}

async fn visibility(State(ctx): State<SharedContext>, Json(req): Json<FlagRequest>) -> Result<Json<EmptyResponse>, ApiError> {
    let mut ctx = ctx.lock().await;
    ctx.set_visibility(EntityId(req.entity_id), req.value).map_err(ApiError::from)?;
    Ok(Json(EmptyResponse {}))
}

async fn transform(State(ctx): State<SharedContext>, Json(req): Json<TransformRequest>) -> Result<Json<EmptyResponse>, ApiError> {
    let mut ctx = ctx.lock().await;
    let transform = Transform { matrix: req.matrix.into() };
    ctx.set_transform(EntityId(req.entity_id), transform).map_err(ApiError::from)?;
    Ok(Json(EmptyResponse {}))
}

async fn render(State(ctx): State<SharedContext>, Json(req): Json<RenderRequest>) -> Result<Json<RenderResponse>, ApiError> {
    let mut ctx = ctx.lock().await;
    let camera = CameraParams { view: req.camera.view.into(), proj: req.camera.proj.into(), viewport: req.camera.viewport.into() };
    ctx.render(&camera).map_err(ApiError::from)?;
    Ok(Json(RenderResponse { frame_id: 0 }))
}

async fn get_state(State(ctx): State<SharedContext>, axum::extract::Path(id): axum::extract::Path<u64>) -> Result<Json<StateResponse>, ApiError> {
    let ctx = ctx.lock().await;
    let state = ctx.get_state(EntityId(id)).map_err(ApiError::from)?;
    Ok(Json(StateResponse {
        visual: VisualPayload::from(state.visual),
        transform: MatrixPayload::from(state.transform.matrix),
        has_mesh: state.has_mesh,
    }))
}

async fn screenshot(State(_ctx): State<SharedContext>) -> Result<Json<ScreenshotResponse>, ApiError> {
    // Placeholder: real implementation should capture a PNG.
    Ok(Json(ScreenshotResponse {
        image_base64: String::new(),
    }))
}

// --- Models & error mapping ---

#[derive(Debug)]
pub struct ApiError {
    status: axum::http::StatusCode,
    body: ErrorBody,
}

#[derive(Debug, serde::Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}

impl ApiError {
    fn new(status: axum::http::StatusCode, code: &'static str, message: String) -> Self {
        Self {
            status,
            body: ErrorBody {
                code: code.to_string(),
                message,
            },
        }
    }
}

impl From<SceneError> for ApiError {
    fn from(err: SceneError) -> Self {
        match err {
            SceneError::UnknownEntity(id) => ApiError::new(axum::http::StatusCode::NOT_FOUND, "UnknownEntity", format!("unknown entity {id}")),
            SceneError::ResourceMissing(msg) => ApiError::new(axum::http::StatusCode::BAD_REQUEST, "ResourceMissing", msg.to_string()),
            SceneError::InvalidState(msg) => ApiError::new(axum::http::StatusCode::BAD_REQUEST, "InvalidState", msg.to_string()),
            SceneError::Io(e) => ApiError::new(axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Io", e.to_string()),
            SceneError::Backend(msg) => ApiError::new(axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Backend", msg.to_string()),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = self.status;
        let body = axum::Json(self.body);
        (status, body).into_response()
    }
}
