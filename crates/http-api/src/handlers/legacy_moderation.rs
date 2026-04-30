use std::collections::HashMap;

use axum::{Json, extract::State, http::HeaderMap};
use evt_domain::UserPreview;
use serde::{Deserialize, Serialize};

use crate::{
    auth::authenticate_request,
    handlers::legacy_access::{ensure_can_view_post, legacy_no_permission},
    response::{ApiEnvelope, HttpApiError, success},
    state::HttpState,
};

#[derive(Debug, Deserialize)]
pub struct IdBody {
    id: i64,
}

#[derive(Debug, Deserialize)]
pub struct VisibilityBody {
    id: i64,
    visibility: i32,
}

#[derive(Debug, Deserialize)]
pub struct ReplyBody {
    comment_id: i64,
    content: String,
    at_user_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct CommentThumbBody {
    tweet_id: i64,
    comment_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct ReplyThumbBody {
    tweet_id: i64,
    comment_id: i64,
    reply_id: i64,
}

#[derive(Debug, Serialize)]
pub struct LockResponse {
    lock_status: i32,
}

#[derive(Debug, Serialize)]
pub struct TopResponse {
    top_status: i32,
}

#[derive(Debug, Serialize)]
pub struct HighlightResponse {
    highlight_status: i32,
}

#[derive(Debug, Serialize)]
pub struct VisibilityResponse {
    visibility_status: i32,
}

pub async fn post_lock(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<IdBody>,
) -> Result<Json<ApiEnvelope<LockResponse>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let post = state.app().get_post(payload.id).await?;
    let current = state.app().get_current_user(&actor).await?;
    if post.user_id != actor.id && !current.is_admin {
        return Err(legacy_no_permission());
    }
    let status = state.app().toggle_post_lock(payload.id).await?;
    Ok(Json(success(LockResponse {
        lock_status: if status { 1 } else { 0 },
    })))
}

pub async fn post_stick(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<IdBody>,
) -> Result<Json<ApiEnvelope<TopResponse>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let post = state.app().get_post(payload.id).await?;
    let current = state.app().get_current_user(&actor).await?;
    if post.user_id != actor.id && !current.is_admin {
        return Err(legacy_no_permission());
    }
    let status = state.app().toggle_post_top(payload.id).await?;
    Ok(Json(success(TopResponse {
        top_status: if status { 1 } else { 0 },
    })))
}

pub async fn post_highlight(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<IdBody>,
) -> Result<Json<ApiEnvelope<HighlightResponse>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let post = state.app().get_post(payload.id).await?;
    let current = state.app().get_current_user(&actor).await?;
    if post.user_id != actor.id && !current.is_admin {
        return Err(legacy_no_permission());
    }
    let status = state.app().toggle_post_essence(payload.id).await?;
    Ok(Json(success(HighlightResponse {
        highlight_status: if status { 1 } else { 0 },
    })))
}

pub async fn post_visibility(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<VisibilityBody>,
) -> Result<Json<ApiEnvelope<VisibilityResponse>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let post = state.app().get_post(payload.id).await?;
    let current = state.app().get_current_user(&actor).await?;
    if post.user_id != actor.id && !current.is_admin {
        return Err(legacy_no_permission());
    }
    let visibility = state
        .app()
        .set_post_visibility(payload.id, payload.visibility)
        .await?;
    Ok(Json(success(VisibilityResponse {
        visibility_status: visibility,
    })))
}

pub async fn comment_highlight(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<IdBody>,
) -> Result<Json<ApiEnvelope<HighlightResponse>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let comment = state.app().get_comment(payload.id).await?;
    let post = state.app().get_post(comment.post_id).await?;
    if post.user_id != actor.id {
        return Err(legacy_no_permission());
    }
    let status = state.app().toggle_comment_essence(payload.id).await?;
    Ok(Json(success(HighlightResponse {
        highlight_status: if status { 1 } else { 0 },
    })))
}

pub async fn comment_reply(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<ReplyBody>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let comment = state.app().get_comment(payload.comment_id).await?;
    let post = state.app().get_post(comment.post_id).await?;
    ensure_can_view_post(state.app(), Some(&actor), &post).await?;
    let reply = state
        .app()
        .create_comment_reply(
            &actor,
            payload.comment_id,
            payload.at_user_id,
            &payload.content,
        )
        .await?;
    let users = state
        .app()
        .batch_user_previews_by_ids(&[reply.user_id, reply.at_user_id])
        .await?;
    Ok(Json(success(reply_payload(&reply, &users))))
}

pub async fn comment_reply_delete(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<IdBody>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    if let Err(err) = state.app().delete_comment_reply(&actor, payload.id).await {
        return match err {
            evt_domain::AppError::Unauthorized(_) => Err(legacy_no_permission()),
            other => Err(other.into()),
        };
    }
    Ok(Json(success(serde_json::json!({}))))
}

pub async fn comment_thumbsup(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<CommentThumbBody>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let post = state.app().get_post(payload.tweet_id).await?;
    ensure_can_view_post(state.app(), Some(&actor), &post).await?;
    state
        .app()
        .toggle_comment_thumb(&actor, payload.tweet_id, payload.comment_id, true)
        .await?;
    Ok(Json(success(serde_json::json!({}))))
}

fn reply_payload(
    reply: &evt_domain::CommentReplySummary,
    users: &HashMap<i64, UserPreview>,
) -> serde_json::Value {
    serde_json::json!({
        "id": reply.id,
        "comment_id": reply.comment_id,
        "user_id": reply.user_id,
        "at_user_id": reply.at_user_id,
        "content": reply.content,
        "ip_loc": "",
        "thumbs_up_count": 0,
        "is_thumbs_up": 0,
        "is_thumbs_down": 0,
        "created_on": reply.created_at.timestamp(),
        "user": compat_user_json(users.get(&reply.user_id), reply.user_id, reply.created_at.timestamp()),
        "at_user": compat_user_json(users.get(&reply.at_user_id), reply.at_user_id, reply.created_at.timestamp())
    })
}

fn compat_user_json(
    user: Option<&UserPreview>,
    user_id: i64,
    created_on: i64,
) -> serde_json::Value {
    if let Some(user) = user {
        serde_json::json!({
            "id": user.id,
            "nickname": user.nickname,
            "username": user.username,
            "avatar": user.avatar,
            "phone": "",
            "activation": "",
            "is_admin": false,
            "is_friend": false,
            "is_following": false,
            "created_on": user.created_at.timestamp(),
            "follows": 0,
            "followings": 0,
            "tweets_count": 0,
            "balance": 0,
            "status": 1
        })
    } else {
        serde_json::json!({
            "id": user_id,
            "nickname": "",
            "username": "",
            "avatar": "",
            "phone": "",
            "activation": "",
            "is_admin": false,
            "is_friend": false,
            "is_following": false,
            "created_on": created_on,
            "follows": 0,
            "followings": 0,
            "tweets_count": 0,
            "balance": 0,
            "status": 1
        })
    }
}

pub async fn comment_thumbsdown(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<CommentThumbBody>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let post = state.app().get_post(payload.tweet_id).await?;
    ensure_can_view_post(state.app(), Some(&actor), &post).await?;
    state
        .app()
        .toggle_comment_thumb(&actor, payload.tweet_id, payload.comment_id, false)
        .await?;
    Ok(Json(success(serde_json::json!({}))))
}

pub async fn reply_thumbsup(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<ReplyThumbBody>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let post = state.app().get_post(payload.tweet_id).await?;
    ensure_can_view_post(state.app(), Some(&actor), &post).await?;
    state
        .app()
        .toggle_reply_thumb(
            &actor,
            payload.tweet_id,
            payload.comment_id,
            payload.reply_id,
            true,
        )
        .await?;
    Ok(Json(success(serde_json::json!({}))))
}

pub async fn reply_thumbsdown(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<ReplyThumbBody>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let post = state.app().get_post(payload.tweet_id).await?;
    ensure_can_view_post(state.app(), Some(&actor), &post).await?;
    state
        .app()
        .toggle_reply_thumb(
            &actor,
            payload.tweet_id,
            payload.comment_id,
            payload.reply_id,
            false,
        )
        .await?;
    Ok(Json(success(serde_json::json!({}))))
}
