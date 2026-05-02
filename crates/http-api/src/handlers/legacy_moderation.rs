use axum::{Json, extract::State, http::HeaderMap};
use evt_domain::AppError;
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
pub struct CommentThumbBody {
    tweet_id: i64,
    comment_id: i64,
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

fn map_legacy_permission_error(error: HttpApiError) -> HttpApiError {
    match error {
        HttpApiError::App(AppError::Unauthorized(_)) => legacy_no_permission(),
        other => other,
    }
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
    let status = match state.app().toggle_post_lock(&actor, payload.id).await {
        Ok(status) => status,
        Err(AppError::Unauthorized(_)) => return Err(legacy_no_permission()),
        Err(other) => return Err(other.into()),
    };
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
    let status = match state.app().toggle_post_top(&actor, payload.id).await {
        Ok(status) => status,
        Err(AppError::Unauthorized(_)) => return Err(legacy_no_permission()),
        Err(other) => return Err(other.into()),
    };
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
    let status = match state.app().toggle_post_essence(&actor, payload.id).await {
        Ok(status) => status,
        Err(AppError::Unauthorized(_)) => return Err(legacy_no_permission()),
        Err(other) => return Err(other.into()),
    };
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
    let visibility = match state
        .app()
        .set_post_visibility(&actor, payload.id, payload.visibility)
        .await
    {
        Ok(visibility) => visibility,
        Err(AppError::Unauthorized(_)) => return Err(legacy_no_permission()),
        Err(other) => return Err(other.into()),
    };
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
    let status = match state.app().toggle_comment_essence(&actor, payload.id).await {
        Ok(status) => status,
        Err(AppError::Unauthorized(_)) => return Err(legacy_no_permission()),
        Err(other) => return Err(other.into()),
    };
    Ok(Json(success(HighlightResponse {
        highlight_status: if status { 1 } else { 0 },
    })))
}

pub async fn comment_thumbsup(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<CommentThumbBody>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let post = state.app().get_post(payload.tweet_id).await?;
    if let Err(err) = ensure_can_view_post(state.app(), Some(&actor), &post).await {
        return Err(map_legacy_permission_error(err));
    }
    if let Err(err) = state
        .app()
        .toggle_comment_thumb(&actor, payload.tweet_id, payload.comment_id, true)
        .await
    {
        return match err {
            AppError::Unauthorized(_) => Err(legacy_no_permission()),
            other => Err(other.into()),
        };
    }
    Ok(Json(success(serde_json::json!({}))))
}

pub async fn comment_thumbsdown(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<CommentThumbBody>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let post = state.app().get_post(payload.tweet_id).await?;
    if let Err(err) = ensure_can_view_post(state.app(), Some(&actor), &post).await {
        return Err(map_legacy_permission_error(err));
    }
    if let Err(err) = state
        .app()
        .toggle_comment_thumb(&actor, payload.tweet_id, payload.comment_id, false)
        .await
    {
        return match err {
            AppError::Unauthorized(_) => Err(legacy_no_permission()),
            other => Err(other.into()),
        };
    }
    Ok(Json(success(serde_json::json!({}))))
}
