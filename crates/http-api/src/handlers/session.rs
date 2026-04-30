use axum::{Json, extract::State};
use evt_domain::{LoginResult, RegisterResult};
use serde::Deserialize;

use crate::{
    response::{ApiEnvelope, HttpApiError, success},
    state::HttpState,
};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

pub async fn register(
    State(state): State<HttpState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<ApiEnvelope<RegisterResult>>, HttpApiError> {
    let resp = state
        .app()
        .register(&payload.username, &payload.password)
        .await?;
    Ok(Json(success(resp)))
}

pub async fn login(
    State(state): State<HttpState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<ApiEnvelope<LoginResult>>, HttpApiError> {
    let resp = state
        .app()
        .login(&payload.username, &payload.password)
        .await?;
    Ok(Json(success(resp)))
}
