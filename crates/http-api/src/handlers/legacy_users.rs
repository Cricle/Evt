use std::collections::HashMap;

use axum::{
    Json,
    extract::{Query, State},
    http::HeaderMap,
};
use paopao_domain::{LegacyPostState, PagedResponse, PostContentItem, PostSummary, UserPreview};
use serde::{Deserialize, Serialize};

use crate::{
    auth::{authenticate_optional_request, authenticate_request},
    handlers::legacy_access::{batch_relation_maps, can_view_post, legacy_visibility},
    response::{ApiEnvelope, HttpApiError, success},
    state::HttpState,
};

#[derive(Debug, Serialize)]
pub(crate) struct CompatPager {
    pub(crate) page: u64,
    pub(crate) page_size: u64,
    pub(crate) total_rows: i64,
}

#[derive(Debug, Serialize)]
pub(crate) struct CompatListResponse<T> {
    pub(crate) list: Vec<T>,
    pub(crate) pager: CompatPager,
}

#[derive(Debug, Serialize, Clone)]
pub(crate) struct CompatUserInfo {
    pub(crate) id: i64,
    pub(crate) nickname: String,
    pub(crate) username: String,
    pub(crate) avatar: String,
    pub(crate) phone: String,
    pub(crate) activation: String,
    pub(crate) is_admin: bool,
    pub(crate) is_friend: bool,
    pub(crate) is_following: bool,
    pub(crate) created_on: i64,
    pub(crate) follows: i64,
    pub(crate) followings: i64,
    pub(crate) tweets_count: i64,
    pub(crate) balance: i64,
    pub(crate) status: i32,
}

#[derive(Debug, Serialize)]
pub struct CompatContactItem {
    pub(crate) user_id: i64,
    pub(crate) username: String,
    pub(crate) nickname: String,
    pub(crate) avatar: String,
    pub(crate) is_following: bool,
    pub(crate) created_on: i64,
}

#[derive(Debug, Serialize, Clone)]
pub(crate) struct CompatPostContentItem {
    id: i64,
    #[serde(rename = "type")]
    content_type: i32,
    post_id: i64,
    content: String,
    sort: i64,
    created_on: i64,
}

#[derive(Debug, Serialize, Clone)]
pub(crate) struct CompatPost {
    id: i64,
    user_id: i64,
    user: CompatUserInfo,
    attachment_price: i64,
    ip_loc: String,
    latest_replied_on: i64,
    created_on: i64,
    upvote_count: i64,
    comment_count: i64,
    collection_count: i64,
    share_count: i64,
    contents: Vec<CompatPostContentItem>,
    tags: String,
    visibility: i32,
    is_lock: i32,
    is_top: i32,
    is_essence: i32,
}

#[derive(Debug, Serialize)]
pub struct SuggestResponse {
    suggest: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct TrendItem {
    nickname: String,
    username: String,
    avatar: String,
    is_fresh: bool,
}

#[derive(Debug, Deserialize)]
pub struct CompatUserProfileQuery {
    username: String,
}

#[derive(Debug, Deserialize)]
pub struct CompatUserPostsQuery {
    username: String,
    style: Option<String>,
    page: Option<u64>,
    page_size: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct CompatFollowListQuery {
    username: String,
    page: Option<u64>,
    page_size: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct CompatSuggestQuery {
    k: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CompatPageQuery {
    pub(crate) page: Option<u64>,
    pub(crate) page_size: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct CompatFollowActionBody {
    user_id: i64,
}

pub async fn user_profile(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Query(query): Query<CompatUserProfileQuery>,
) -> Result<Json<ApiEnvelope<CompatUserInfo>>, HttpApiError> {
    let actor = authenticate_optional_request(state.app(), &headers).await?;
    let profile = state.app().get_user_profile(&query.username).await?;
    let (is_following, is_friend) = match actor {
        Some(ref actor) if actor.id != profile.id => (
            state.app().is_following(actor.id, profile.id).await?,
            state.app().is_friend(actor.id, profile.id).await?,
        ),
        _ => (false, false),
    };

    Ok(Json(success(CompatUserInfo {
        id: profile.id,
        nickname: profile.nickname.clone(),
        username: profile.username,
        avatar: profile.avatar,
        phone: profile.phone_number.unwrap_or_default(),
        activation: profile.activation_code,
        is_admin: profile.is_admin,
        is_friend,
        is_following,
        created_on: profile.created_at.timestamp(),
        follows: profile.followings_count,
        followings: profile.followers_count,
        tweets_count: profile.posts_count,
        balance: profile.balance,
        status: if profile.status.eq_ignore_ascii_case("disabled") {
            2
        } else {
            1
        },
    })))
}

pub async fn user_posts(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Query(query): Query<CompatUserPostsQuery>,
) -> Result<Json<ApiEnvelope<CompatListResponse<CompatPost>>>, HttpApiError> {
    let actor = authenticate_optional_request(state.app(), &headers).await?;
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let style = query.style.as_deref().unwrap_or("post");

    let posts = match style {
        "post" => {
            state
                .app()
                .list_user_posts(&query.username, page, page_size)
                .await?
        }
        "star" => {
            state
                .app()
                .list_user_star_posts(&query.username, page, page_size)
                .await?
        }
        "highlight" => {
            state
                .app()
                .list_user_highlight_posts(&query.username, page, page_size)
                .await?
        }
        "media" => {
            state
                .app()
                .list_user_media_posts(&query.username, page, page_size)
                .await?
        }
        "comment" => {
            state
                .app()
                .list_user_commented_posts(&query.username, page, page_size)
                .await?
        }
        _ => {
            return Ok(Json(success(CompatListResponse {
                list: Vec::new(),
                pager: CompatPager {
                    page,
                    page_size,
                    total_rows: 0,
                },
            })));
        }
    };
    let profile = state.app().get_user_profile(&query.username).await?;
    let (is_following, is_friend) = match actor {
        Some(ref actor) if actor.id != profile.id => (
            state.app().is_following(actor.id, profile.id).await?,
            state.app().is_friend(actor.id, profile.id).await?,
        ),
        _ => (false, false),
    };
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
        batch_relation_maps(state.app(), actor.as_ref(), &author_ids).await?;
    let use_profile_author = matches!(style, "post" | "highlight" | "media");

    Ok(Json(success(CompatListResponse {
        list: posts
            .items
            .into_iter()
            .filter(|post| {
                can_view_post(
                    actor.as_ref(),
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
                let user = if use_profile_author && post.user_id == profile.id {
                    compat_user_from_profile(&profile, is_following, is_friend)
                } else {
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
                    )
                };
                let mut item = to_compat_post(
                    &post,
                    grouped_contents.get(&post.id).map(Vec::as_slice),
                    user,
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

pub async fn user_follows(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Query(query): Query<CompatFollowListQuery>,
) -> Result<Json<ApiEnvelope<CompatListResponse<CompatContactItem>>>, HttpApiError> {
    let actor = authenticate_optional_request(state.app(), &headers).await?;
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let users = state
        .app()
        .list_following_previews(&query.username, page, page_size)
        .await?;
    let statuses = following_status_map(state.app(), actor.as_ref(), &users.items).await?;

    Ok(Json(success(contact_list_response(users, &statuses))))
}

pub async fn user_followings(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Query(query): Query<CompatFollowListQuery>,
) -> Result<Json<ApiEnvelope<CompatListResponse<CompatContactItem>>>, HttpApiError> {
    let actor = authenticate_optional_request(state.app(), &headers).await?;
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let users = state
        .app()
        .list_follower_previews(&query.username, page, page_size)
        .await?;
    let statuses = following_status_map(state.app(), actor.as_ref(), &users.items).await?;

    Ok(Json(success(contact_list_response(users, &statuses))))
}

pub async fn follow_user(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<CompatFollowActionBody>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    state
        .app()
        .follow_user_by_id(&actor, payload.user_id)
        .await?;
    Ok(Json(success(serde_json::json!({}))))
}

pub async fn unfollow_user(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<CompatFollowActionBody>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    state
        .app()
        .unfollow_user_by_id(&actor, payload.user_id)
        .await?;
    Ok(Json(success(serde_json::json!({}))))
}

pub async fn suggest_users(
    State(state): State<HttpState>,
    Query(query): Query<CompatSuggestQuery>,
) -> Result<Json<ApiEnvelope<SuggestResponse>>, HttpApiError> {
    let keyword = query.k.unwrap_or_default();
    let suggest = state.app().suggest_usernames(&keyword, 8).await?;
    Ok(Json(success(SuggestResponse { suggest })))
}

pub async fn trends_index(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Query(query): Query<CompatPageQuery>,
) -> Result<Json<ApiEnvelope<CompatListResponse<TrendItem>>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let users = state
        .app()
        .list_following_previews(&actor.username, page, page_size)
        .await?;

    Ok(Json(success(CompatListResponse {
        list: users
            .items
            .into_iter()
            .map(|user| TrendItem {
                nickname: user.nickname,
                username: user.username,
                avatar: user.avatar,
                is_fresh: false,
            })
            .collect(),
        pager: CompatPager {
            page: users.page,
            page_size: users.page_size,
            total_rows: users.total,
        },
    })))
}

pub(crate) fn contact_list_response(
    users: PagedResponse<UserPreview>,
    statuses: &HashMap<i64, bool>,
) -> CompatListResponse<CompatContactItem> {
    CompatListResponse {
        list: users
            .items
            .into_iter()
            .map(|user| CompatContactItem {
                user_id: user.id,
                username: user.username.clone(),
                nickname: user.nickname,
                avatar: user.avatar,
                is_following: statuses.get(&user.id).copied().unwrap_or(false),
                created_on: user.created_at.timestamp(),
            })
            .collect(),
        pager: CompatPager {
            page: users.page,
            page_size: users.page_size,
            total_rows: users.total,
        },
    }
}

pub(crate) async fn following_status_map(
    app: &paopao_infra::AppContext,
    actor: Option<&paopao_domain::UserIdentity>,
    users: &[UserPreview],
) -> Result<HashMap<i64, bool>, HttpApiError> {
    let Some(actor) = actor else {
        return Ok(HashMap::new());
    };
    let user_ids = users.iter().map(|item| item.id).collect::<Vec<_>>();
    app.batch_following_status(actor.id, &user_ids)
        .await
        .map_err(Into::into)
}

pub(crate) fn compat_user_from_profile(
    profile: &paopao_domain::UserProfile,
    is_following: bool,
    is_friend: bool,
) -> CompatUserInfo {
    CompatUserInfo {
        id: profile.id,
        nickname: profile.nickname.clone(),
        username: profile.username.clone(),
        avatar: profile.avatar.clone(),
        phone: profile.phone_number.clone().unwrap_or_default(),
        activation: profile.activation_code.clone(),
        is_admin: profile.is_admin,
        is_friend,
        is_following,
        created_on: profile.created_at.timestamp(),
        follows: profile.followings_count,
        followings: profile.followers_count,
        tweets_count: profile.posts_count,
        balance: profile.balance,
        status: if profile.status.eq_ignore_ascii_case("disabled") {
            2
        } else {
            1
        },
    }
}

pub(crate) fn compat_user_from_post(
    preview: Option<&UserPreview>,
    user_id: i64,
    username: &str,
    created_on: i64,
    is_following: bool,
    is_friend: bool,
) -> CompatUserInfo {
    CompatUserInfo {
        id: preview.map(|item| item.id).unwrap_or(user_id),
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

pub(crate) fn apply_post_state(post: &mut CompatPost, state: Option<&LegacyPostState>) {
    let Some(state) = state else {
        return;
    };
    post.attachment_price = state.attachment_price;
    post.visibility = state.visibility;
    post.is_lock = if state.is_lock { 1 } else { 0 };
    post.is_top = if state.is_top { 1 } else { 0 };
    post.is_essence = if state.is_essence { 1 } else { 0 };
}

pub(crate) fn to_compat_post(
    post: &PostSummary,
    contents: Option<&[PostContentItem]>,
    user: CompatUserInfo,
) -> CompatPost {
    let created_on = post.created_at.timestamp();
    CompatPost {
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
        contents: to_compat_post_contents(post.id, created_on, &post.content, contents),
        tags: post.tags.clone(),
        visibility: 0,
        is_lock: 0,
        is_top: 0,
        is_essence: 0,
    }
}

fn to_compat_post_contents(
    post_id: i64,
    created_on: i64,
    raw: &str,
    contents: Option<&[PostContentItem]>,
) -> Vec<CompatPostContentItem> {
    if let Some(items) = contents {
        return items
            .iter()
            .map(|item| CompatPostContentItem {
                id: item.id,
                content_type: item.content_type,
                post_id: item.post_id,
                content: item.content.clone(),
                sort: item.sort,
                created_on: item.created_at.timestamp(),
            })
            .collect();
    }

    vec![CompatPostContentItem {
        id: 1,
        content_type: 2,
        post_id,
        content: raw.to_string(),
        sort: 100,
        created_on,
    }]
}

pub(crate) fn group_post_contents(
    contents: Vec<PostContentItem>,
) -> HashMap<i64, Vec<PostContentItem>> {
    let mut grouped = HashMap::new();
    for item in contents {
        grouped
            .entry(item.post_id)
            .or_insert_with(Vec::new)
            .push(item);
    }
    grouped
}

#[cfg(test)]
mod tests {
    use paopao_domain::{UserPreview, UserProfile};

    use super::{compat_user_from_post, compat_user_from_profile};

    #[test]
    fn compat_profile_counts_keep_legacy_field_names() {
        let profile = UserProfile {
            id: 7,
            username: "bob".into(),
            nickname: "B".into(),
            avatar: "/avatar.png".into(),
            is_admin: false,
            is_friend: false,
            is_following: false,
            phone_number: None,
            activation_code: String::new(),
            balance: 12,
            status: "active".into(),
            created_at: "2024-01-01T00:00:00Z".parse().unwrap(),
            posts_count: 5,
            comments_count: 0,
            followings_count: 11,
            followers_count: 13,
        };

        let user = compat_user_from_profile(&profile, true, false);
        assert_eq!(user.follows, 11);
        assert_eq!(user.followings, 13);
        assert!(user.is_following);
        assert!(!user.is_friend);
    }

    #[test]
    fn compat_post_user_falls_back_to_post_metadata() {
        let created_on = 1234;
        let user = compat_user_from_post(None, 9, "charlie", created_on, true, true);
        assert_eq!(user.id, 9);
        assert_eq!(user.username, "charlie");
        assert_eq!(user.nickname, "charlie");
        assert_eq!(user.created_on, created_on);
        assert!(user.is_following);
        assert!(user.is_friend);
    }

    #[test]
    fn compat_post_user_prefers_preview_fields_when_present() {
        let preview = UserPreview {
            id: 3,
            username: "dora".into(),
            nickname: "Dora".into(),
            avatar: "/dora.png".into(),
            created_at: "2024-01-01T00:00:00Z".parse().unwrap(),
        };

        let user = compat_user_from_post(Some(&preview), 99, "ignored", 1, false, false);
        assert_eq!(user.id, 3);
        assert_eq!(user.username, "dora");
        assert_eq!(user.nickname, "Dora");
        assert_eq!(user.avatar, "/dora.png");
    }
}
