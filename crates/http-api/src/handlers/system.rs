use axum::{Json, extract::State};
use paopao_domain::{SiteProfile, VersionInfo};
use serde::Serialize;

use crate::{
    response::{ApiEnvelope, HttpApiError, success},
    state::HttpState,
};

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    status: &'static str,
}

pub async fn healthz(
    State(state): State<HttpState>,
) -> Result<Json<ApiEnvelope<HealthResponse>>, HttpApiError> {
    state.app().healthcheck().await?;
    Ok(Json(success(HealthResponse { status: "ok" })))
}

pub async fn openapi_spec() -> Json<serde_json::Value> {
    Json(
        serde_json::from_str(include_str!("../../../../docs/openapi.json"))
            .expect("embedded openapi.json must be valid"),
    )
}

pub async fn version_root(State(state): State<HttpState>) -> Json<ApiEnvelope<VersionInfo>> {
    Json(success(state.app().version()))
}

pub async fn site_version(State(state): State<HttpState>) -> Json<ApiEnvelope<VersionInfo>> {
    Json(success(state.app().version()))
}

pub async fn site_profile(State(state): State<HttpState>) -> Json<ApiEnvelope<SiteProfile>> {
    Json(success(state.app().site_profile()))
}
