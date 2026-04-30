use std::collections::HashMap;

use axum::{
    Json,
    extract::{Query, State},
    http::HeaderMap,
};
use paopao_domain::{LegacyMessageSummary, UserPreview};
use serde::{Deserialize, Serialize};

use crate::{
    auth::authenticate_request,
    handlers::legacy_users::{CompatListResponse, CompatPager, CompatUserInfo},
    response::{ApiEnvelope, HttpApiError, legacy_error, success},
    state::HttpState,
};

#[derive(Debug, Deserialize)]
pub struct LegacyMessagesQuery {
    style: Option<String>,
    page: Option<u64>,
    page_size: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct LegacyReadMessageBody {
    id: i64,
}

#[derive(Debug, Deserialize)]
pub struct LegacyWhisperBody {
    user_id: i64,
    content: String,
}

#[derive(Debug, Serialize)]
pub struct LegacyUnreadCountResponse {
    count: i64,
}

#[derive(Debug, Serialize)]
pub struct LegacyMessageItem {
    id: i64,
    #[serde(rename = "type")]
    message_type: i32,
    brief: String,
    content: String,
    is_read: i32,
    sender_user_id: i64,
    sender_user: CompatUserInfo,
    receiver_user_id: i64,
    receiver_user: CompatUserInfo,
    post_id: i64,
    post: serde_json::Value,
    comment_id: i64,
    comment: serde_json::Value,
    reply_id: i64,
    replay: serde_json::Value,
    created_on: i64,
}

pub async fn user_messages(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Query(query): Query<LegacyMessagesQuery>,
) -> Result<Json<ApiEnvelope<CompatListResponse<LegacyMessageItem>>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let style = query.style.as_deref().unwrap_or("all");

    let messages = state
        .app()
        .list_legacy_messages(&actor, style, page, page_size)
        .await?;

    let user_ids = messages
        .items
        .iter()
        .flat_map(|item| [item.sender_user_id, item.receiver_user_id])
        .collect::<Vec<_>>();
    let users = load_user_map(state.app(), &user_ids).await?;
    let following_status = state
        .app()
        .batch_following_status(actor.id, &user_ids)
        .await?;

    Ok(Json(success(CompatListResponse {
        list: messages
            .items
            .into_iter()
            .map(|item| to_legacy_message_item(item, &users, &following_status))
            .collect(),
        pager: CompatPager {
            page: messages.page,
            page_size: messages.page_size,
            total_rows: messages.total,
        },
    })))
}

pub async fn message_read(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<LegacyReadMessageBody>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    state.app().mark_message_read(&actor, payload.id).await?;
    Ok(Json(success(serde_json::json!({}))))
}

pub async fn message_read_all(
    State(state): State<HttpState>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    state.app().mark_all_messages_read(&actor).await?;
    Ok(Json(success(serde_json::json!({}))))
}

pub async fn unread_message_count(
    State(state): State<HttpState>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<LegacyUnreadCountResponse>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let count = state.app().unread_legacy_message_count(&actor).await?;
    Ok(Json(success(LegacyUnreadCountResponse { count })))
}

pub async fn user_whisper(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<LegacyWhisperBody>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let receiver = state.app().get_user_preview_by_id(payload.user_id).await?;
    if actor.id == receiver.id {
        return Err(legacy_error(
            axum::http::StatusCode::BAD_REQUEST,
            50004,
            "不允许给自己发送私信",
        ));
    }
    if payload.content.trim().is_empty() {
        return Err(legacy_error(
            axum::http::StatusCode::BAD_REQUEST,
            50003,
            "私信发送失败",
        ));
    }
    state
        .app()
        .send_legacy_message(
            actor.id,
            receiver.id,
            4,
            "给你发送新私信了",
            payload.content.trim(),
            0,
            0,
            0,
        )
        .await?;
    Ok(Json(success(serde_json::json!({}))))
}

fn to_legacy_message_item(
    item: LegacyMessageSummary,
    users: &HashMap<i64, CompatUserInfo>,
    following_status: &HashMap<i64, bool>,
) -> LegacyMessageItem {
    LegacyMessageItem {
        id: item.id,
        message_type: item.message_type,
        brief: item.brief,
        content: item.content,
        is_read: if item.is_read { 1 } else { 0 },
        sender_user_id: item.sender_user_id,
        sender_user: users
            .get(&item.sender_user_id)
            .cloned()
            .map(|mut user| {
                user.is_following = following_status
                    .get(&item.sender_user_id)
                    .copied()
                    .unwrap_or(false);
                user
            })
            .unwrap_or_else(|| empty_user(item.sender_user_id)),
        receiver_user_id: item.receiver_user_id,
        receiver_user: users
            .get(&item.receiver_user_id)
            .cloned()
            .map(|mut user| {
                user.is_following = following_status
                    .get(&item.receiver_user_id)
                    .copied()
                    .unwrap_or(false);
                user
            })
            .unwrap_or_else(|| empty_user(item.receiver_user_id)),
        post_id: item.post_id,
        post: if item.post_id > 0 {
            serde_json::json!({ "id": item.post_id })
        } else {
            serde_json::json!({})
        },
        comment_id: item.comment_id,
        comment: if item.comment_id > 0 {
            serde_json::json!({ "id": item.comment_id })
        } else {
            serde_json::json!({})
        },
        reply_id: item.reply_id,
        replay: serde_json::json!({}),
        created_on: item.created_at.timestamp(),
    }
}

async fn load_user_map(
    app: &paopao_infra::AppContext,
    user_ids: &[i64],
) -> Result<HashMap<i64, CompatUserInfo>, HttpApiError> {
    Ok(app
        .batch_user_previews_by_ids(user_ids)
        .await?
        .into_values()
        .map(|user| (user.id, compat_user_from_preview(user)))
        .collect())
}

fn compat_user_from_preview(user: UserPreview) -> CompatUserInfo {
    CompatUserInfo {
        id: user.id,
        nickname: user.nickname,
        username: user.username,
        avatar: user.avatar,
        phone: String::new(),
        activation: String::new(),
        is_admin: false,
        is_friend: false,
        is_following: false,
        created_on: user.created_at.timestamp(),
        follows: 0,
        followings: 0,
        tweets_count: 0,
        balance: 0,
        status: 1,
    }
}

fn empty_user(user_id: i64) -> CompatUserInfo {
    CompatUserInfo {
        id: user_id,
        nickname: String::new(),
        username: String::new(),
        avatar: String::new(),
        phone: String::new(),
        activation: String::new(),
        is_admin: false,
        is_friend: false,
        is_following: false,
        created_on: 0,
        follows: 0,
        followings: 0,
        tweets_count: 0,
        balance: 0,
        status: 1,
    }
}
