use axum::{Json, extract::State, response::Html};
use evt_domain::{AppError, SiteProfile, VersionInfo};
use serde::Serialize;
use std::fs;

use crate::{
    response::{ApiEnvelope, HttpApiError, success},
    state::HttpState,
    web_assets::resolve_spa_index_path,
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

pub async fn spa_shell(State(state): State<HttpState>) -> Result<Html<String>, HttpApiError> {
    let index_path = resolve_spa_index_path(&state.app().settings().web.dist_dir)
        .map_err(HttpApiError::from)?;
    let html = fs::read_to_string(&index_path)
        .map_err(|err| HttpApiError::from(AppError::Internal(format!("read spa shell: {err}"))))?;
    Ok(Html(html))
}
