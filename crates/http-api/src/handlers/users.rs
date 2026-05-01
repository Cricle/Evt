use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
};
use evt_domain::{
    CurrentUser, FollowActionResult, PagedResponse, PostSummary, UserProfile, UserSummary,
};
use serde::Deserialize;

use crate::{
    auth::{authenticate_optional_request, authenticate_request},
    pagination::PageQuery,
    response::{ApiEnvelope, HttpApiError, success},
    state::HttpState,
};

#[derive(Debug, Deserialize)]
pub struct ChangeNicknameRequest {
    username: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    old_password: String,
    new_password: String,
}

pub async fn current_user(
    State(state): State<HttpState>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<CurrentUser>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let current_user = state.app().get_current_user(&actor).await?;
    Ok(Json(success(current_user)))
}

pub async fn change_nickname(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<ChangeNicknameRequest>,
) -> Result<Json<ApiEnvelope<CurrentUser>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let current_user = state
        .app()
        .change_nickname(&actor, &payload.username)
        .await?;
    Ok(Json(success(current_user)))
}

pub async fn change_password(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    state
        .app()
        .change_password(&actor, &payload.old_password, &payload.new_password)
        .await?;
    Ok(Json(success(serde_json::json!({}))))
}

pub async fn user_profile(
    State(state): State<HttpState>,
    Path(username): Path<String>,
) -> Result<Json<ApiEnvelope<UserProfile>>, HttpApiError> {
    let profile = state.app().get_user_profile(&username).await?;
    Ok(Json(success(profile)))
}

pub async fn user_posts(
    State(state): State<HttpState>,
    Path(username): Path<String>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Result<Json<ApiEnvelope<PagedResponse<PostSummary>>>, HttpApiError> {
    let actor = authenticate_optional_request(state.app(), &headers).await?;
    let (page, page_size) = query.normalized();
    let posts = state
        .app()
        .list_user_posts_for_viewer(actor.as_ref(), &username, page, page_size)
        .await?;
    Ok(Json(success(posts)))
}

pub async fn user_followers(
    State(state): State<HttpState>,
    Path(username): Path<String>,
    Query(query): Query<PageQuery>,
) -> Result<Json<ApiEnvelope<PagedResponse<UserSummary>>>, HttpApiError> {
    let (page, page_size) = query.normalized();
    let followers = state
        .app()
        .list_followers(&username, page, page_size)
        .await?;
    Ok(Json(success(followers)))
}

pub async fn user_followings(
    State(state): State<HttpState>,
    Path(username): Path<String>,
    Query(query): Query<PageQuery>,
) -> Result<Json<ApiEnvelope<PagedResponse<UserSummary>>>, HttpApiError> {
    let (page, page_size) = query.normalized();
    let followings = state
        .app()
        .list_followings(&username, page, page_size)
        .await?;
    Ok(Json(success(followings)))
}

pub async fn follow_user(
    State(state): State<HttpState>,
    Path(username): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<FollowActionResult>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let result = state.app().follow_user(&actor, &username).await?;
    Ok(Json(success(result)))
}

pub async fn unfollow_user(
    State(state): State<HttpState>,
    Path(username): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<FollowActionResult>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let result = state.app().unfollow_user(&actor, &username).await?;
    Ok(Json(success(result)))
}
