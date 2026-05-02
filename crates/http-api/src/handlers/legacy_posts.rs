use axum::{
    Json,
    extract::{Query, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};

use crate::{
    auth::authenticate_request,
    handlers::legacy_access::ensure_can_view_post,
    response::{ApiEnvelope, HttpApiError, success},
    state::HttpState,
};

#[derive(Debug, Serialize)]
pub struct ToggleStatusResponse {
    status: bool,
}

#[derive(Debug, Deserialize)]
pub struct PostStatusQuery {
    id: i64,
}

pub async fn get_post_star(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Query(query): Query<PostStatusQuery>,
) -> Result<Json<ApiEnvelope<ToggleStatusResponse>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let post = state.app().get_post(query.id).await?;
    ensure_can_view_post(state.app(), Some(&actor), &post).await?;
    let status = state.app().has_starred_post(&actor, query.id).await?;
    Ok(Json(success(ToggleStatusResponse { status })))
}

pub async fn post_star(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<PostStatusQuery>,
) -> Result<Json<ApiEnvelope<ToggleStatusResponse>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let post = state.app().get_post(payload.id).await?;
    ensure_can_view_post(state.app(), Some(&actor), &post).await?;
    let status = state.app().toggle_post_star(&actor, payload.id).await?;
    Ok(Json(success(ToggleStatusResponse { status })))
}
