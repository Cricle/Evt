use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
};
use evt_domain::{CommentSummary, PagedResponse, PostSummary, TogglePostReactionResult};
use serde::Deserialize;

use crate::{
    auth::{authenticate_optional_request, authenticate_request},
    handlers::legacy_access::ensure_can_view_post,
    pagination::PageQuery,
    response::{ApiEnvelope, HttpApiError, success},
    state::HttpState,
};

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePostRequest {
    content: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    content: String,
}

#[derive(Debug, Deserialize)]
pub struct ToggleReactionRequest {
    emoji: String,
}

pub async fn feed(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Result<Json<ApiEnvelope<PagedResponse<PostSummary>>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let (page, page_size) = query.normalized();
    let feed = state.app().list_feed(&actor, page, page_size).await?;
    Ok(Json(success(feed)))
}

#[allow(dead_code)]
pub async fn list_posts(
    State(state): State<HttpState>,
    Query(query): Query<PageQuery>,
) -> Result<Json<ApiEnvelope<PagedResponse<PostSummary>>>, HttpApiError> {
    let (page, page_size) = query.normalized();
    let posts = state.app().list_posts(page, page_size).await?;
    Ok(Json(success(posts)))
}

pub async fn get_post(
    State(state): State<HttpState>,
    Path(post_id): Path<i64>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<PostSummary>>, HttpApiError> {
    let viewer = authenticate_optional_request(state.app(), &headers).await?;
    let post = state.app().get_post(post_id).await?;
    ensure_can_view_post(state.app(), viewer.as_ref(), &post).await?;
    Ok(Json(success(post)))
}

pub async fn create_post(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<CreatePostRequest>,
) -> Result<Json<ApiEnvelope<PostSummary>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let post = state.app().create_post(&actor, &payload.content).await?;
    Ok(Json(success(post)))
}

pub async fn update_post(
    State(state): State<HttpState>,
    Path(post_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<UpdatePostRequest>,
) -> Result<Json<ApiEnvelope<PostSummary>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let post = state
        .app()
        .update_post(&actor, post_id, &payload.content)
        .await?;
    Ok(Json(success(post)))
}

pub async fn delete_post(
    State(state): State<HttpState>,
    Path(post_id): Path<i64>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    state.app().delete_post(&actor, post_id).await?;
    Ok(Json(success(serde_json::json!({}))))
}

pub async fn list_comments(
    State(state): State<HttpState>,
    Path(post_id): Path<i64>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Result<Json<ApiEnvelope<PagedResponse<CommentSummary>>>, HttpApiError> {
    let viewer = authenticate_optional_request(state.app(), &headers).await?;
    let post = state.app().get_post(post_id).await?;
    ensure_can_view_post(state.app(), viewer.as_ref(), &post).await?;
    let (page, page_size) = query.normalized();
    let comments = state.app().list_comments(post_id, page, page_size).await?;
    Ok(Json(success(comments)))
}

pub async fn create_comment(
    State(state): State<HttpState>,
    Path(post_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<Json<ApiEnvelope<CommentSummary>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let post = state.app().get_post(post_id).await?;
    ensure_can_view_post(state.app(), Some(&actor), &post).await?;
    let comment = state
        .app()
        .create_comment(&actor, post_id, &payload.content)
        .await?;
    Ok(Json(success(comment)))
}

pub async fn delete_comment(
    State(state): State<HttpState>,
    Path(comment_id): Path<i64>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    state.app().delete_comment(&actor, comment_id).await?;
    Ok(Json(success(serde_json::json!({}))))
}

pub async fn list_post_reactions(
    State(state): State<HttpState>,
    Path(post_id): Path<i64>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<Vec<evt_domain::PostReactionSummary>>>, HttpApiError> {
    let viewer = authenticate_optional_request(state.app(), &headers).await?;
    let post = state.app().get_post(post_id).await?;
    ensure_can_view_post(state.app(), viewer.as_ref(), &post).await?;
    let reactions = state.app().list_post_reactions(viewer.as_ref(), post_id).await?;
    Ok(Json(success(reactions)))
}

pub async fn toggle_post_reaction(
    State(state): State<HttpState>,
    Path(post_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<ToggleReactionRequest>,
) -> Result<Json<ApiEnvelope<TogglePostReactionResult>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let post = state.app().get_post(post_id).await?;
    ensure_can_view_post(state.app(), Some(&actor), &post).await?;
    let result = state
        .app()
        .toggle_post_reaction(&actor, post_id, &payload.emoji)
        .await?;
    Ok(Json(success(result)))
}
