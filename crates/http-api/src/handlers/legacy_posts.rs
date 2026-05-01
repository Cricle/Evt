use axum::{
    Json,
    extract::{Query, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};

use crate::{
    auth::authenticate_request,
    handlers::legacy_access::{batch_relation_maps, ensure_can_view_post},
    handlers::legacy_users::{
        CompatListResponse, CompatPageQuery, CompatPager, CompatPost, apply_post_state,
        compat_user_from_post, group_post_contents, to_compat_post,
    },
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

pub async fn get_post_collection(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Query(query): Query<PostStatusQuery>,
) -> Result<Json<ApiEnvelope<ToggleStatusResponse>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let post = state.app().get_post(query.id).await?;
    ensure_can_view_post(state.app(), Some(&actor), &post).await?;
    let status = state.app().has_collected_post(&actor, query.id).await?;
    Ok(Json(success(ToggleStatusResponse { status })))
}

pub async fn post_collection(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<PostStatusQuery>,
) -> Result<Json<ApiEnvelope<ToggleStatusResponse>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let post = state.app().get_post(payload.id).await?;
    ensure_can_view_post(state.app(), Some(&actor), &post).await?;
    let status = state
        .app()
        .toggle_post_collection(&actor, payload.id)
        .await?;
    Ok(Json(success(ToggleStatusResponse { status })))
}

pub async fn user_collections(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Query(query): Query<CompatPageQuery>,
) -> Result<Json<ApiEnvelope<CompatListResponse<CompatPost>>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let posts = state
        .app()
        .list_user_collections_for_viewer(&actor, &actor, page, page_size)
        .await?;
    let post_ids = posts.items.iter().map(|item| item.id).collect::<Vec<_>>();
    let grouped_contents = group_post_contents(state.app().list_post_contents(&post_ids).await?);
    let post_states = state.app().legacy_post_states_by_ids(&post_ids).await?;
    let author_ids = posts
        .items
        .iter()
        .map(|item| item.user_id)
        .collect::<Vec<_>>();
    let previews = state.app().batch_user_previews_by_ids(&author_ids).await?;
    let (following_status, friend_status) =
        batch_relation_maps(state.app(), Some(&actor), &author_ids).await?;

    Ok(Json(success(CompatListResponse {
        list: posts
            .items
            .into_iter()
            .map(|post| {
                let mut item = to_compat_post(
                    &post,
                    grouped_contents.get(&post.id).map(Vec::as_slice),
                    compat_user_from_post(
                        previews.get(&post.user_id),
                        post.user_id,
                        &post.username,
                        post.created_at.timestamp(),
                        following_status
                            .get(&post.user_id)
                            .copied()
                            .unwrap_or(false),
                        friend_status.get(&post.user_id).copied().unwrap_or(false),
                    ),
                );
                apply_post_state(&mut item, post_states.get(&post.id));
                item
            })
            .collect(),
        pager: CompatPager {
            page: posts.page,
            page_size: posts.page_size,
            total_rows: posts.total,
        },
    })))
}
