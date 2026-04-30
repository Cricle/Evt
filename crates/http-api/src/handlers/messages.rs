use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
};
use paopao_domain::{MessageSummary, PagedResponse, UnreadCount};
use serde::Deserialize;

use crate::{
    auth::authenticate_request,
    pagination::PageQuery,
    response::{ApiEnvelope, HttpApiError, success},
    state::HttpState,
};

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    receiver_username: String,
    content: String,
}

pub async fn list_messages(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Result<Json<ApiEnvelope<PagedResponse<MessageSummary>>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let (page, page_size) = query.normalized();
    let messages = state.app().list_messages(&actor, page, page_size).await?;
    Ok(Json(success(messages)))
}

pub async fn unread_message_count(
    State(state): State<HttpState>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<UnreadCount>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let unread = state.app().unread_message_count(&actor).await?;
    Ok(Json(success(unread)))
}

pub async fn send_message(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<SendMessageRequest>,
) -> Result<Json<ApiEnvelope<MessageSummary>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let message = state
        .app()
        .send_message(&actor, &payload.receiver_username, &payload.content)
        .await?;
    Ok(Json(success(message)))
}

pub async fn mark_message_read(
    State(state): State<HttpState>,
    Path(message_id): Path<i64>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    state.app().mark_message_read(&actor, message_id).await?;
    Ok(Json(success(serde_json::json!({}))))
}

pub async fn mark_all_messages_read(
    State(state): State<HttpState>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    state.app().mark_all_messages_read(&actor).await?;
    Ok(Json(success(serde_json::json!({}))))
}
