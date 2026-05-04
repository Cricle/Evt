use std::collections::HashMap;

use axum::{
    Json,
    extract::{Multipart, Query, State},
    http::HeaderMap,
};
use evt_domain::{
    AppError, AttachmentSummary, CommentContentItem, CommentSummary, CreateContentInput,
    CurrentUser, PostContentItem, PostReactionSummary, PostSummary, UserProfile,
};
use serde::{Deserialize, Serialize};

use crate::{
    auth::{authenticate_optional_request, authenticate_request},
    handlers::legacy_access::{
        batch_relation_maps, can_view_post, ensure_can_view_post, legacy_no_permission,
        legacy_visibility,
    },
    response::{ApiEnvelope, HttpApiError, success},
    state::HttpState,
};

#[derive(Debug, Serialize)]
pub struct LegacyPager {
    page: u64,
    page_size: u64,
    total_rows: i64,
}

#[derive(Debug, Serialize)]
pub struct LegacyListResponse<T> {
    list: Vec<T>,
    pager: LegacyPager,
}

#[derive(Debug, Serialize, Clone)]
pub struct LegacyUserInfo {
    id: i64,
    nickname: String,
    username: String,
    avatar: String,
    phone: String,
    activation: String,
    is_admin: bool,
    is_friend: bool,
    is_following: bool,
    created_on: i64,
    follows: i64,
    followings: i64,
    tweets_count: i64,
    balance: i64,
    status: i32,
}

#[derive(Debug, Serialize, Clone)]
pub struct LegacyPostContentItem {
    id: i64,
    #[serde(rename = "type")]
    content_type: i32,
    post_id: i64,
    content: String,
    sort: i64,
    created_on: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct LegacyPost {
    id: i64,
    user_id: i64,
    user: LegacyUserInfo,
    attachment_price: i64,
    ip_loc: String,
    latest_replied_on: i64,
    created_on: i64,
    upvote_count: i64,
    comment_count: i64,
    collection_count: i64,
    share_count: i64,
    contents: Vec<LegacyPostContentItem>,
    tags: String,
    reactions: Vec<PostReactionSummary>,
    visibility: i32,
    is_lock: i32,
    is_top: i32,
    is_essence: i32,
}

#[derive(Debug, Serialize, Clone)]
pub struct LegacyComment {
    id: i64,
    post_id: i64,
    user_id: i64,
    user: LegacyUserInfo,
    contents: Vec<LegacyCommentContentItem>,
    replies: Vec<serde_json::Value>,
    ip_loc: String,
    is_essence: i32,
    thumbs_up_count: i64,
    is_thumbs_up: i32,
    is_thumbs_down: i32,
    created_on: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct LegacyCommentContentItem {
    id: i64,
    comment_id: i64,
    user_id: i64,
    #[serde(rename = "type")]
    content_type: i32,
    content: String,
    sort: i64,
    created_on: i64,
}

#[derive(Debug, Serialize)]
pub struct LegacyUploadAttachmentResponse {
    user_id: i64,
    file_size: i64,
    img_width: i32,
    img_height: i32,
    #[serde(rename = "type")]
    attachment_type: i32,
    content: String,
}

#[derive(Debug, Deserialize)]
pub struct LegacyPostQuery {
    id: i64,
}

#[derive(Debug, Deserialize)]
pub struct LegacyPostsQuery {
    query: Option<String>,
    #[serde(rename = "type")]
    query_type: Option<String>,
    space_slug: Option<String>,
    page: Option<u64>,
    page_size: Option<u64>,
    style: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LegacyCommentListQuery {
    id: i64,
    style: Option<String>,
    page: Option<u64>,
    page_size: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct LegacyDeleteBody {
    id: i64,
}

#[derive(Debug, Deserialize)]
pub struct LegacyCreatePostBody {
    space_slug: Option<String>,
    contents: Vec<LegacyIncomingContent>,
    tags: Vec<String>,
    users: Vec<String>,
    attachment_price: i64,
    visibility: i32,
}

#[derive(Debug, Deserialize)]
pub struct LegacyCreateCommentBody {
    post_id: i64,
    contents: Vec<LegacyIncomingContent>,
    users: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LegacyIncomingContent {
    content: String,
    #[serde(rename = "type")]
    content_type: i32,
    sort: i64,
}

pub async fn user_info(
    State(state): State<HttpState>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<LegacyUserInfo>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let current = state.app().get_current_user(&actor).await?;
    let profile = state
        .app()
        .get_user_profile_for_viewer(Some(&actor), &actor.username)
        .await?;
    Ok(Json(success(to_legacy_user_info(&current, Some(&profile)))))
}

pub async fn list_posts(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Query(query): Query<LegacyPostsQuery>,
) -> Result<Json<ApiEnvelope<LegacyListResponse<LegacyPost>>>, HttpApiError> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let style = query.style.as_deref().unwrap_or("newest");
    let viewer = authenticate_optional_request(state.app(), &headers).await?;
    let space_slug = query.space_slug.as_deref();

    let posts = if style.eq_ignore_ascii_case("following") {
        if let Some(actor) = viewer.clone() {
            state
                .app()
                .list_feed_in_space(&actor, space_slug, page, page_size)
                .await?
        } else {
            state
                .app()
                .list_posts_in_space(viewer.as_ref(), space_slug, page, page_size)
                .await?
        }
    } else if style.eq_ignore_ascii_case("hots") {
        state
            .app()
            .list_hot_posts_in_space(viewer.as_ref(), space_slug, page, page_size)
            .await?
    } else if style.eq_ignore_ascii_case("search") {
        state
            .app()
            .search_posts_in_space(
                viewer.as_ref(),
                space_slug,
                query.query.as_deref().unwrap_or_default(),
                query.query_type.as_deref(),
                page,
                page_size,
            )
            .await?
    } else {
        state
            .app()
            .list_posts_in_space(viewer.as_ref(), space_slug, page, page_size)
            .await?
    };

    let post_ids = posts.items.iter().map(|item| item.id).collect::<Vec<_>>();
    let author_ids = posts
        .items
        .iter()
        .map(|item| item.user_id)
        .collect::<Vec<_>>();
    let grouped_contents = group_post_contents(state.app().list_post_contents(&post_ids).await?);
    let post_states = state.app().legacy_post_states_by_ids(&post_ids).await?;
    let post_reactions = state
        .app()
        .list_post_reactions_by_post_ids(viewer.as_ref(), &post_ids)
        .await?;
    let users = state.app().batch_user_previews_by_ids(&author_ids).await?;
    let (following_status, friend_status) =
        batch_relation_maps(state.app(), viewer.as_ref(), &author_ids).await?;

    Ok(Json(success(LegacyListResponse {
        list: posts
            .items
            .into_iter()
            .filter(|post| {
                can_view_post(
                    viewer.as_ref(),
                    post.user_id,
                    legacy_visibility(post_states.get(&post.id)),
                    following_status
                        .get(&post.user_id)
                        .copied()
                        .unwrap_or(false),
                    friend_status.get(&post.user_id).copied().unwrap_or(false),
                )
            })
            .map(|post| {
                let mut item = to_legacy_post(
                    &post,
                    grouped_contents.get(&post.id).map(Vec::as_slice),
                    following_status
                        .get(&post.user_id)
                        .copied()
                        .unwrap_or(false),
                    false,
                    users.get(&post.user_id),
                    post_reactions.get(&post.id).cloned().unwrap_or_default(),
                );
                apply_post_state(&mut item, post_states.get(&post.id));
                item
            })
            .collect(),
        pager: LegacyPager {
            page: posts.page,
            page_size: posts.page_size,
            total_rows: posts.total,
        },
    })))
}

pub async fn get_post(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Query(query): Query<LegacyPostQuery>,
) -> Result<Json<ApiEnvelope<LegacyPost>>, HttpApiError> {
    let viewer = authenticate_optional_request(state.app(), &headers).await?;
    let post = state.app().get_post(query.id).await?;
    ensure_can_view_post(state.app(), viewer.as_ref(), &post).await?;
    let grouped_contents = group_post_contents(state.app().list_post_contents(&[post.id]).await?);
    let states = state.app().legacy_post_states_by_ids(&[post.id]).await?;
    let reactions = state
        .app()
        .list_post_reactions(viewer.as_ref(), post.id)
        .await?;
    let users = state
        .app()
        .batch_user_previews_by_ids(&[post.user_id])
        .await?;
    let (is_following, is_friend) =
        super::legacy_access::relation_status(state.app(), viewer.as_ref(), post.user_id).await?;
    let mut item = to_legacy_post(
        &post,
        grouped_contents.get(&query.id).map(Vec::as_slice),
        is_following,
        is_friend,
        users.get(&post.user_id),
        reactions,
    );
    apply_post_state(&mut item, states.get(&post.id));
    Ok(Json(success(item)))
}

pub async fn create_post(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<LegacyCreatePostBody>,
) -> Result<Json<ApiEnvelope<LegacyPost>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let _ = &payload.users;
    let contents = payload
        .contents
        .into_iter()
        .map(into_create_content_input)
        .collect::<Vec<_>>();
    let post = state
        .app()
        .create_legacy_post_in_space(
            &actor,
            payload.space_slug.as_deref(),
            &contents,
            &payload.tags,
            payload.attachment_price,
            payload.visibility,
        )
        .await?;
    let grouped_contents = group_post_contents(state.app().list_post_contents(&[post.id]).await?);
    let states = state.app().legacy_post_states_by_ids(&[post.id]).await?;
    let users = state
        .app()
        .batch_user_previews_by_ids(&[post.user_id])
        .await?;
    let mut item = to_legacy_post(
        &post,
        grouped_contents.get(&post.id).map(Vec::as_slice),
        false,
        false,
        users.get(&post.user_id),
        Vec::new(),
    );
    apply_post_state(&mut item, states.get(&post.id));
    Ok(Json(success(item)))
}

pub async fn delete_post(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<LegacyDeleteBody>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    if let Err(err) = state.app().delete_post(&actor, payload.id).await {
        return match err {
            AppError::Unauthorized(_) => Err(legacy_no_permission()),
            other => Err(other.into()),
        };
    }
    Ok(Json(success(serde_json::json!({}))))
}

pub async fn list_comments(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Query(query): Query<LegacyCommentListQuery>,
) -> Result<Json<ApiEnvelope<LegacyListResponse<LegacyComment>>>, HttpApiError> {
    let viewer = authenticate_optional_request(state.app(), &headers).await?;
    let post = state.app().get_post(query.id).await?;
    ensure_can_view_post(state.app(), viewer.as_ref(), &post).await?;
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let comments = state
        .app()
        .list_legacy_comments(
            query.id,
            query.style.as_deref().unwrap_or("default"),
            page,
            page_size,
        )
        .await?;
    let comment_ids = comments
        .items
        .iter()
        .map(|item| item.id)
        .collect::<Vec<_>>();
    let grouped_contents =
        group_comment_contents(state.app().list_comment_contents(&comment_ids).await?);
    let comment_states = state
        .app()
        .legacy_comment_states_by_ids(&comment_ids)
        .await?;
    let comment_user_ids = comments
        .items
        .iter()
        .map(|item| item.user_id)
        .collect::<Vec<_>>();
    let users = state
        .app()
        .batch_user_previews_by_ids(&comment_user_ids)
        .await?;
    let comment_thumb_counts = state.app().comment_thumb_counts(&comment_ids).await?;
    let reaction_statuses = match viewer.as_ref() {
        Some(viewer) => {
            state
                .app()
                .reaction_status_map(viewer.id, &comment_ids)
                .await?
        }
        None => HashMap::new(),
    };

    Ok(Json(success(LegacyListResponse {
        list: comments
            .items
            .into_iter()
            .map(|comment| {
                to_legacy_comment(
                    &comment,
                    grouped_contents.get(&comment.id).map(Vec::as_slice),
                    comment_states.get(&comment.id),
                    comment_thumb_counts
                        .get(&comment.id)
                        .copied()
                        .unwrap_or_default(),
                    reaction_statuses
                        .get(&(0, comment.id))
                        .copied()
                        .unwrap_or((false, false)),
                    &users,
                )
            })
            .collect(),
        pager: LegacyPager {
            page: comments.page,
            page_size: comments.page_size,
            total_rows: comments.total,
        },
    })))
}

pub async fn create_comment(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<LegacyCreateCommentBody>,
) -> Result<Json<ApiEnvelope<LegacyComment>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let post = state.app().get_post(payload.post_id).await?;
    ensure_can_view_post(state.app(), Some(&actor), &post).await?;
    let _ = &payload.users;
    let contents = payload
        .contents
        .into_iter()
        .map(into_create_content_input)
        .collect::<Vec<_>>();
    let comment = state
        .app()
        .create_legacy_comment(&actor, payload.post_id, &contents)
        .await?;
    let grouped_contents =
        group_comment_contents(state.app().list_comment_contents(&[comment.id]).await?);
    let states = state
        .app()
        .legacy_comment_states_by_ids(&[comment.id])
        .await?;
    let users = state
        .app()
        .batch_user_previews_by_ids(&[comment.user_id])
        .await?;
    Ok(Json(success(to_legacy_comment(
        &comment,
        grouped_contents.get(&comment.id).map(Vec::as_slice),
        states.get(&comment.id),
        0,
        (false, false),
        &users,
    ))))
}

pub async fn delete_comment(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<LegacyDeleteBody>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    if let Err(err) = state.app().delete_comment(&actor, payload.id).await {
        return match err {
            AppError::Unauthorized(_) => Err(legacy_no_permission()),
            other => Err(other.into()),
        };
    }
    Ok(Json(success(serde_json::json!({}))))
}

pub async fn upload_attachment(
    State(state): State<HttpState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<ApiEnvelope<LegacyUploadAttachmentResponse>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let mut upload_type = "attachment".to_string();

    while let Some(field) = multipart.next_field().await.map_err(|err| {
        HttpApiError::from(AppError::Validation(format!(
            "invalid multipart body: {err}"
        )))
    })? {
        if field.name() == Some("type") {
            upload_type = field.text().await.unwrap_or_else(|_| "attachment".into());
            continue;
        }

        if field.name() != Some("file") {
            continue;
        }

        let file_name = field.file_name().unwrap_or("file").to_string();
        let content_type = field.content_type().map(ToOwned::to_owned);
        let bytes = field.bytes().await.map_err(|err| {
            HttpApiError::from(AppError::Validation(format!(
                "read multipart field failed: {err}"
            )))
        })?;

        let attachment = state
            .app()
            .upload_attachment_by_kind(&actor, upload_type.as_str(), &file_name, content_type.as_deref(), &bytes)
            .await?;

        return Ok(Json(success(to_legacy_attachment_upload_response(
            attachment,
            upload_type.as_str(),
        ))));
    }

    Err(AppError::Validation("multipart field `file` is required".into()).into())
}

fn to_legacy_attachment_upload_response(
    attachment: AttachmentSummary,
    upload_type: &str,
) -> LegacyUploadAttachmentResponse {
    let content = match upload_type {
        "public/image" | "public/video" => format!("/v1/media/{}", attachment.id),
        _ => format!("/v1/attachments/{}", attachment.id),
    };

    LegacyUploadAttachmentResponse {
        user_id: attachment.user_id,
        file_size: attachment.size_bytes,
        img_width: 0,
        img_height: 0,
        attachment_type: legacy_attachment_type(upload_type),
        content,
    }
}

fn to_legacy_post(
    post: &PostSummary,
    contents: Option<&[PostContentItem]>,
    is_following: bool,
    is_friend: bool,
    user: Option<&evt_domain::UserPreview>,
    reactions: Vec<PostReactionSummary>,
) -> LegacyPost {
    let user = fallback_user(
        user,
        post.user_id,
        &post.username,
        post.created_at.timestamp(),
        is_following,
        is_friend,
    );
    let created_on = post.created_at.timestamp();
    LegacyPost {
        id: post.id,
        user_id: post.user_id,
        user,
        attachment_price: 0,
        ip_loc: String::new(),
        latest_replied_on: created_on,
        created_on,
        upvote_count: post.upvote_count,
        comment_count: post.comments_count,
        collection_count: post.collection_count,
        share_count: 0,
        contents: to_legacy_post_contents(post.id, created_on, &post.content, contents),
        tags: post.tags.clone(),
        reactions,
        visibility: 0,
        is_lock: 0,
        is_top: 0,
        is_essence: 0,
    }
}

fn to_legacy_comment(
    comment: &CommentSummary,
    contents: Option<&[CommentContentItem]>,
    state: Option<&evt_domain::LegacyCommentState>,
    thumbs_up_count: i64,
    self_status: (bool, bool),
    users: &HashMap<i64, evt_domain::UserPreview>,
) -> LegacyComment {
    let created_on = comment.created_at.timestamp();
    LegacyComment {
        id: comment.id,
        post_id: comment.post_id,
        user_id: comment.user_id,
        user: fallback_user(
            users.get(&comment.user_id),
            comment.user_id,
            &comment.username,
            created_on,
            false,
            false,
        ),
        contents: to_legacy_comment_contents(
            comment.id,
            comment.user_id,
            created_on,
            &comment.content,
            contents,
        ),
        replies: Vec::new(),
        ip_loc: String::new(),
        is_essence: if state.map(|item| item.is_essence).unwrap_or(false) {
            1
        } else {
            0
        },
        thumbs_up_count,
        is_thumbs_up: if self_status.0 { 1 } else { 0 },
        is_thumbs_down: if self_status.1 { 1 } else { 0 },
        created_on,
    }
}

fn to_legacy_user_info(current: &CurrentUser, profile: Option<&UserProfile>) -> LegacyUserInfo {
    LegacyUserInfo {
        id: current.id,
        nickname: current.nickname.clone(),
        username: current.username.clone(),
        avatar: current.avatar.clone(),
        phone: current.phone_number.clone().unwrap_or_default(),
        activation: current.activation_code.clone(),
        is_admin: current.is_admin,
        is_friend: profile.map(|item| item.is_friend).unwrap_or(false),
        is_following: profile.map(|item| item.is_following).unwrap_or(false),
        created_on: current.created_at.timestamp(),
        follows: profile
            .map(|item| item.followings_count)
            .unwrap_or_default(),
        followings: profile.map(|item| item.followers_count).unwrap_or_default(),
        tweets_count: profile.map(|item| item.posts_count).unwrap_or_default(),
        balance: current.balance,
        status: if current.status.eq_ignore_ascii_case("disabled") {
            2
        } else {
            1
        },
    }
}

fn fallback_user(
    preview: Option<&evt_domain::UserPreview>,
    id: i64,
    username: &str,
    created_on: i64,
    is_following: bool,
    is_friend: bool,
) -> LegacyUserInfo {
    LegacyUserInfo {
        id: preview.map(|item| item.id).unwrap_or(id),
        nickname: preview
            .map(|item| item.nickname.clone())
            .unwrap_or_else(|| username.to_string()),
        username: preview
            .map(|item| item.username.clone())
            .unwrap_or_else(|| username.to_string()),
        avatar: preview.map(|item| item.avatar.clone()).unwrap_or_default(),
        phone: String::new(),
        activation: String::new(),
        is_admin: false,
        is_friend,
        is_following,
        created_on: preview
            .map(|item| item.created_at.timestamp())
            .unwrap_or(created_on),
        follows: 0,
        followings: 0,
        tweets_count: 0,
        balance: 0,
        status: 1,
    }
}

fn apply_post_state(post: &mut LegacyPost, state: Option<&evt_domain::LegacyPostState>) {
    let Some(state) = state else {
        return;
    };
    post.attachment_price = state.attachment_price;
    post.visibility = state.visibility;
    post.is_lock = if state.is_lock { 1 } else { 0 };
    post.is_top = if state.is_top { 1 } else { 0 };
    post.is_essence = if state.is_essence { 1 } else { 0 };
}

fn to_legacy_post_contents(
    post_id: i64,
    created_on: i64,
    raw: &str,
    contents: Option<&[PostContentItem]>,
) -> Vec<LegacyPostContentItem> {
    if let Some(items) = contents {
        return items
            .iter()
            .map(|item| LegacyPostContentItem {
                id: item.id,
                content_type: item.content_type,
                post_id: item.post_id,
                content: normalize_legacy_content_url(item.content_type, &item.content),
                sort: item.sort,
                created_on: item.created_at.timestamp(),
            })
            .collect();
    }

    vec![LegacyPostContentItem {
        id: 1,
        content_type: 2,
        post_id,
        content: raw.to_string(),
        sort: 100,
        created_on,
    }]
}

fn to_legacy_comment_contents(
    comment_id: i64,
    user_id: i64,
    created_on: i64,
    raw: &str,
    contents: Option<&[CommentContentItem]>,
) -> Vec<LegacyCommentContentItem> {
    if let Some(items) = contents {
        return items
            .iter()
            .map(|item| LegacyCommentContentItem {
                id: item.id,
                comment_id: item.comment_id,
                user_id: item.user_id,
                content_type: item.content_type,
                content: normalize_legacy_content_url(item.content_type, &item.content),
                sort: item.sort,
                created_on: item.created_at.timestamp(),
            })
            .collect();
    }

    vec![LegacyCommentContentItem {
        id: 1,
        comment_id,
        user_id,
        content_type: 2,
        content: raw.to_string(),
        sort: 100,
        created_on,
    }]
}

fn normalize_legacy_content_url(content_type: i32, content: &str) -> String {
    match content_type {
        3 | 4 => {
            if let Some(attachment_id) = parse_attachment_id_from_url(content) {
                return format!("/v1/media/{attachment_id}");
            }
            content.to_string()
        }
        _ => content.to_string(),
    }
}

fn parse_attachment_id_from_url(content: &str) -> Option<i64> {
    let trimmed = content.trim();
    if !trimmed.contains("/v1/attachments/") {
        return None;
    }
    trimmed
        .split('?')
        .next()?
        .trim_end_matches('/')
        .rsplit('/')
        .next()?
        .parse()
        .ok()
}

fn legacy_attachment_type(upload_type: &str) -> i32 {
    match upload_type {
        "public/image" => 1,
        "public/video" => 2,
        _ => 3,
    }
}

fn into_create_content_input(item: LegacyIncomingContent) -> CreateContentInput {
    CreateContentInput {
        content: item.content,
        content_type: item.content_type,
        sort: item.sort,
    }
}

fn group_post_contents(contents: Vec<PostContentItem>) -> HashMap<i64, Vec<PostContentItem>> {
    let mut grouped = HashMap::new();
    for item in contents {
        grouped
            .entry(item.post_id)
            .or_insert_with(Vec::new)
            .push(item);
    }
    grouped
}

fn group_comment_contents(
    contents: Vec<CommentContentItem>,
) -> HashMap<i64, Vec<CommentContentItem>> {
    let mut grouped = HashMap::new();
    for item in contents {
        grouped
            .entry(item.comment_id)
            .or_insert_with(Vec::new)
            .push(item);
    }
    grouped
}

#[cfg(test)]
mod tests {
    use super::{normalize_legacy_content_url, parse_attachment_id_from_url};

    #[test]
    fn legacy_image_and_video_content_urls_are_rewritten_to_public_media_routes() {
        assert_eq!(
            normalize_legacy_content_url(3, "/v1/attachments/11"),
            "/v1/media/11"
        );
        assert_eq!(
            normalize_legacy_content_url(
                4,
                "http://115.191.9.128:8008/v1/attachments/12?x-oss-process=image/resize,m_fill,w_300"
            ),
            "/v1/media/12"
        );
    }

    #[test]
    fn non_media_content_urls_are_left_unchanged() {
        assert_eq!(
            normalize_legacy_content_url(7, "/v1/attachments/21"),
            "/v1/attachments/21"
        );
        assert_eq!(
            normalize_legacy_content_url(6, "https://evt.example"),
            "https://evt.example"
        );
    }

    #[test]
    fn parses_attachment_id_from_legacy_urls_with_query_strings() {
        assert_eq!(
            parse_attachment_id_from_url(
                "http://115.191.9.128:8008/v1/attachments/11?x-oss-process=image/resize,m_fill,w_300"
            ),
            Some(11)
        );
        assert_eq!(parse_attachment_id_from_url("/v1/media/11"), None);
    }
}
