use std::sync::Arc;

use axum::{body::Body, http::Request};
use tower::ServiceExt;

use crate::{scene::tessellation::TessParams, server::{command_server, models::ShapePayload}, scene::mesh::{MeshData, Vertex}};

fn sample_mesh() -> MeshData {
    MeshData {
        vertices: vec![
            Vertex { position: (0.0, 0.0, 0.0).into(), normal: (0.0, 0.0, 1.0).into(), uv: None },
            Vertex { position: (1.0, 0.0, 0.0).into(), normal: (0.0, 0.0, 1.0).into(), uv: None },
            Vertex { position: (0.0, 1.0, 0.0).into(), normal: (0.0, 0.0, 1.0).into(), uv: None },
        ],
        indices: vec![0, 1, 2],
    }
}

#[tokio::test]
async fn http_submit_then_state() {
    let ctx = Arc::new(tokio::sync::Mutex::new(crate::scene::context::SceneContext::new()));
    let app = command_server(ctx.clone());

    let req_body = serde_json::to_vec(&crate::server::models::SubmitEntityRequest {
        shape: ShapePayload { mesh: sample_mesh() },
        tess_params: Some(TessParams::default()),
        entity_id: None,
    }).unwrap();

    let response = app
        .clone()
        .oneshot(Request::post("/api/entity").header("content-type", "application/json").body(Body::from(req_body)).unwrap())
        .await
        .unwrap();

    assert!(response.status().is_success());
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created: crate::server::models::SubmitEntityResponse = serde_json::from_slice(&bytes).unwrap();

    let state_resp = app
        .clone()
        .oneshot(Request::get(format!("/api/state/{}", created.entity_id)).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert!(state_resp.status().is_success());
    let bytes = axum::body::to_bytes(state_resp.into_body(), usize::MAX).await.unwrap();
    let state: crate::server::models::StateResponse = serde_json::from_slice(&bytes).unwrap();
    assert!(state.has_mesh);
    assert_eq!(state.visual.visible, false);
}
