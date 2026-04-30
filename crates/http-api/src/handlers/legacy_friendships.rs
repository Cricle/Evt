use axum::{
    Json,
    extract::{Query, State},
    http::HeaderMap,
};
use serde::Deserialize;

use crate::{
    auth::authenticate_request,
    handlers::legacy_users::{CompatContactItem, CompatListResponse, CompatPageQuery, CompatPager},
    response::{ApiEnvelope, HttpApiError, success},
    state::HttpState,
};

#[derive(Debug, Deserialize)]
pub struct FriendActionBody {
    user_id: i64,
    greetings: Option<String>,
}

pub async fn friend_requesting(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<FriendActionBody>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    state
        .app()
        .request_friend(
            &actor,
            payload.user_id,
            payload.greetings.as_deref().unwrap_or_default(),
        )
        .await?;
    Ok(Json(success(serde_json::json!({}))))
}

pub async fn friend_add(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<FriendActionBody>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    state.app().add_friend(&actor, payload.user_id).await?;
    Ok(Json(success(serde_json::json!({}))))
}

pub async fn friend_reject(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<FriendActionBody>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    state.app().reject_friend(&actor, payload.user_id).await?;
    Ok(Json(success(serde_json::json!({}))))
}

pub async fn friend_delete(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<FriendActionBody>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    state.app().delete_friend(&actor, payload.user_id).await?;
    Ok(Json(success(serde_json::json!({}))))
}

pub async fn user_contacts(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Query(query): Query<CompatPageQuery>,
) -> Result<Json<ApiEnvelope<CompatListResponse<CompatContactItem>>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let users = state
        .app()
        .list_friend_contacts(&actor, page, page_size)
        .await?;

    Ok(Json(success(CompatListResponse {
        list: users
            .items
            .into_iter()
            .map(|user| CompatContactItem {
                user_id: user.id,
                username: user.username,
                nickname: user.nickname,
                avatar: user.avatar,
                is_following: false,
                created_on: user.created_at.timestamp(),
            })
            .collect(),
        pager: CompatPager {
            page: users.page,
            page_size: users.page_size,
            total_rows: users.total,
        },
    })))
}
