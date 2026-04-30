use axum::{
    Json,
    extract::{Query, State},
    http::HeaderMap,
};
use std::collections::HashMap;

use evt_domain::{TagSummary, UserPreview};
use serde::{Deserialize, Serialize};

use crate::{
    auth::{authenticate_optional_request, authenticate_request},
    handlers::legacy_users::CompatUserInfo,
    response::{ApiEnvelope, HttpApiError, success},
    state::HttpState,
};

#[derive(Debug, Serialize)]
pub struct TopicListResponse {
    topics: Vec<TopicItem>,
    extral_topics: Vec<TopicItem>,
}

#[derive(Debug, Serialize)]
pub struct TopicItem {
    id: i64,
    user_id: i64,
    user: CompatUserInfo,
    tag: String,
    quote_num: i64,
    created_on: i64,
    is_following: i32,
    is_top: i32,
    is_pin: i32,
}

#[derive(Debug, Serialize)]
pub struct SuggestResponse {
    suggest: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct StickTopicResponse {
    top_status: i32,
}

#[derive(Debug, Serialize)]
pub struct PinTopicResponse {
    pin_status: i32,
}

#[derive(Debug, Deserialize)]
pub struct TopicListQuery {
    #[serde(rename = "type")]
    tag_type: String,
    num: u64,
    extral_num: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct SuggestTagsQuery {
    k: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TopicActionBody {
    topic_id: i64,
}

pub async fn list_tags(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Query(query): Query<TopicListQuery>,
) -> Result<Json<ApiEnvelope<TopicListResponse>>, HttpApiError> {
    let actor = authenticate_optional_request(state.app(), &headers).await?;
    let (topics, extral_topics) = state
        .app()
        .list_tags(
            &query.tag_type,
            query.num.max(1).min(100),
            query.extral_num.unwrap_or(0).min(100),
            actor.as_ref(),
        )
        .await?;
    let user_ids = topics
        .iter()
        .chain(extral_topics.iter())
        .map(|item| item.user_id)
        .collect::<Vec<_>>();
    let previews = state.app().batch_user_previews_by_ids(&user_ids).await?;

    Ok(Json(success(TopicListResponse {
        topics: topics
            .into_iter()
            .map(|item| to_topic_item(item, &previews))
            .collect(),
        extral_topics: extral_topics
            .into_iter()
            .map(|item| to_topic_item(item, &previews))
            .collect(),
    })))
}

pub async fn suggest_tags(
    State(state): State<HttpState>,
    Query(query): Query<SuggestTagsQuery>,
) -> Result<Json<ApiEnvelope<SuggestResponse>>, HttpApiError> {
    let suggest = state
        .app()
        .suggest_tags(query.k.as_deref().unwrap_or_default(), 8)
        .await?;
    Ok(Json(success(SuggestResponse { suggest })))
}

pub async fn follow_tag(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<TopicActionBody>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    state.app().follow_tag(&actor, payload.topic_id).await?;
    Ok(Json(success(serde_json::json!({}))))
}

pub async fn unfollow_tag(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<TopicActionBody>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    state.app().unfollow_tag(&actor, payload.topic_id).await?;
    Ok(Json(success(serde_json::json!({}))))
}

pub async fn stick_tag(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<TopicActionBody>,
) -> Result<Json<ApiEnvelope<StickTopicResponse>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let status = state.app().toggle_tag_top(&actor, payload.topic_id).await?;
    Ok(Json(success(StickTopicResponse {
        top_status: if status { 1 } else { 0 },
    })))
}

pub async fn pin_tag(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<TopicActionBody>,
) -> Result<Json<ApiEnvelope<PinTopicResponse>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let status = state.app().toggle_tag_pin(&actor, payload.topic_id).await?;
    Ok(Json(success(PinTopicResponse {
        pin_status: if status { 1 } else { 0 },
    })))
}

fn to_topic_item(item: TagSummary, previews: &HashMap<i64, UserPreview>) -> TopicItem {
    let preview = previews.get(&item.user_id);
    TopicItem {
        id: item.id,
        user_id: item.user_id,
        user: CompatUserInfo {
            id: preview.map(|user| user.id).unwrap_or(item.user_id),
            nickname: preview
                .map(|user| user.nickname.clone())
                .unwrap_or_else(|| item.username.clone()),
            username: preview
                .map(|user| user.username.clone())
                .unwrap_or_else(|| item.username.clone()),
            avatar: preview.map(|user| user.avatar.clone()).unwrap_or_default(),
            phone: String::new(),
            activation: String::new(),
            is_admin: false,
            is_friend: false,
            is_following: false,
            created_on: preview
                .map(|user| user.created_at.timestamp())
                .unwrap_or(item.created_at.timestamp()),
            follows: 0,
            followings: 0,
            tweets_count: 0,
            balance: 0,
            status: 1,
        },
        tag: item.tag,
        quote_num: item.quote_num,
        created_on: item.created_at.timestamp(),
        is_following: if item.is_following { 1 } else { 0 },
        is_top: if item.is_top { 1 } else { 0 },
        is_pin: if item.is_pin { 1 } else { 0 },
    }
}
