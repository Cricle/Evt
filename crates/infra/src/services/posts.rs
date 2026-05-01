use evt_domain::{
    AppError, CommentContentItem, CommentSummary, CreateContentInput, LegacyPostState,
    PagedResponse, PostContentItem, PostSummary, TogglePostReactionResult, UserIdentity,
};

use crate::AppContext;
use crate::repository::group_post_reactions;

const LEGACY_CONTENT_PREFIX: &str = "__EVT_LEGACY_CONTENT__:";
const USER_POST_FILTER_FETCH_LIMIT: u64 = 10_000;

impl AppContext {
    async fn resolve_space_id_for_posts(
        &self,
        viewer: Option<&UserIdentity>,
        space_slug: Option<&str>,
    ) -> Result<i64, AppError> {
        Ok(self.resolve_space(viewer, space_slug).await?.id)
    }

    pub async fn create_post_with_contents(
        &self,
        actor: &UserIdentity,
        contents: &[CreateContentInput],
    ) -> Result<PostSummary, AppError> {
        let space_id = self.default_space_id().await?;
        self.create_post_with_contents_and_tags(actor, space_id, contents, &[])
            .await
    }

    pub async fn create_post_with_contents_and_tags(
        &self,
        actor: &UserIdentity,
        space_id: i64,
        contents: &[CreateContentInput],
        tags: &[String],
    ) -> Result<PostSummary, AppError> {
        let site = self.site_profile_snapshot();
        validate_contents(contents, site.default_tweet_max_length as usize)?;
        validate_media_permissions(contents, &site)?;
        let summary = summary_text(contents);
        let normalized_tags = normalize_tags(tags);
        let tags_value = normalized_tags.join(",");
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(crate::repository::map_db_error)?;
        let post_id = self
            .posts
            .create_tx(&mut tx, space_id, actor.id, &summary, &tags_value)
            .await?;

        for item in contents {
            self.posts
                .create_content_tx(
                    &mut tx,
                    post_id,
                    actor.id,
                    &item.content,
                    item.content_type,
                    item.sort,
                )
                .await?;
        }

        if !normalized_tags.is_empty() {
            self.tags
                .record_tags_tx(&mut tx, space_id, actor.id, &normalized_tags)
                .await?;
        }

        tx.commit().await.map_err(crate::repository::map_db_error)?;
        let post = self
            .posts
            .find_by_id(post_id)
            .await?
            .ok_or_else(|| AppError::Internal("created post cannot be loaded".into()))?;

        self.posts.sync_search_document(post.id).await?;

        Ok(post)
    }

    pub async fn create_post(
        &self,
        actor: &UserIdentity,
        content: &str,
    ) -> Result<PostSummary, AppError> {
        let site = self.site_profile_snapshot();
        let content = content.trim();
        let plain_length = normalized_post_length(content)?;
        if plain_length == 0 {
            return Err(AppError::Validation("post content cannot be empty".into()));
        }
        if plain_length > site.default_tweet_max_length as usize {
            return Err(AppError::Validation(format!(
                "post content exceeds max length {}",
                site.default_tweet_max_length
            )));
        }

        let space_id = self.default_space_id().await?;
        let post = self.posts.create(space_id, actor.id, content, "").await?;
        self.posts.sync_search_document(post.id).await?;
        Ok(post)
    }

    pub async fn list_posts(
        &self,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        self.list_posts_in_space(None, None, page, page_size).await
    }

    pub async fn list_posts_in_space(
        &self,
        viewer: Option<&UserIdentity>,
        space_slug: Option<&str>,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        let space_id = self.resolve_space_id_for_posts(viewer, space_slug).await?;
        self.posts.list(space_id, page, page_size).await
    }

    pub async fn list_hot_posts(
        &self,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        self.list_hot_posts_in_space(None, None, page, page_size)
            .await
    }

    pub async fn list_hot_posts_in_space(
        &self,
        viewer: Option<&UserIdentity>,
        space_slug: Option<&str>,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        let space_id = self.resolve_space_id_for_posts(viewer, space_slug).await?;
        self.posts.list_hot(space_id, page, page_size).await
    }

    pub async fn search_posts(
        &self,
        query: &str,
        query_type: Option<&str>,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        self.search_posts_in_space(None, None, query, query_type, page, page_size)
            .await
    }

    pub async fn search_posts_in_space(
        &self,
        viewer: Option<&UserIdentity>,
        space_slug: Option<&str>,
        query: &str,
        query_type: Option<&str>,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        let space_id = self.resolve_space_id_for_posts(viewer, space_slug).await?;
        self.posts
            .search(space_id, query, query_type, page, page_size)
            .await
    }

    pub async fn list_feed(
        &self,
        actor: &UserIdentity,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        self.list_feed_in_space(actor, None, page, page_size).await
    }

    pub async fn list_feed_in_space(
        &self,
        actor: &UserIdentity,
        space_slug: Option<&str>,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        let space_id = self
            .resolve_space_id_for_posts(Some(actor), space_slug)
            .await?;
        self.posts
            .list_feed(space_id, actor.id, page, page_size)
            .await
    }

    pub async fn get_post(&self, post_id: i64) -> Result<PostSummary, AppError> {
        self.posts
            .find_by_id(post_id)
            .await?
            .ok_or_else(|| AppError::NotFound("post not found".into()))
    }

    pub async fn list_post_contents(
        &self,
        post_ids: &[i64],
    ) -> Result<Vec<PostContentItem>, AppError> {
        self.posts.list_contents_by_post_ids(post_ids).await
    }

    pub async fn update_post(
        &self,
        actor: &UserIdentity,
        post_id: i64,
        content: &str,
    ) -> Result<PostSummary, AppError> {
        let site = self.site_profile_snapshot();
        let existing = self.get_post(post_id).await?;
        if existing.user_id != actor.id {
            return Err(AppError::Unauthorized(
                "cannot edit another user's post".into(),
            ));
        }

        let content = content.trim();
        let plain_length = normalized_post_length(content)?;
        if plain_length == 0 {
            return Err(AppError::Validation("post content cannot be empty".into()));
        }
        if plain_length > site.default_tweet_max_length as usize {
            return Err(AppError::Validation(format!(
                "post content exceeds max length {}",
                site.default_tweet_max_length
            )));
        }

        let post = self
            .posts
            .update_content(post_id, content)
            .await?
            .ok_or_else(|| AppError::Internal("updated post cannot be loaded".into()))?;
        self.posts.sync_search_document(post.id).await?;
        Ok(post)
    }

    pub async fn delete_post(&self, actor: &UserIdentity, post_id: i64) -> Result<(), AppError> {
        let existing = self.get_post(post_id).await?;
        let current = self.get_current_user(actor).await?;
        if existing.user_id != actor.id && !current.is_admin {
            return Err(AppError::Unauthorized(
                "cannot delete another user's post".into(),
            ));
        }
        self.posts.delete_search_document(post_id).await?;
        self.posts.delete_by_id(post_id).await
    }

    pub async fn list_user_posts(
        &self,
        username: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        let space_id = self.default_space_id().await?;
        self.posts
            .list_by_username(space_id, username, page, page_size)
            .await
    }

    pub async fn list_user_posts_for_viewer(
        &self,
        viewer: Option<&UserIdentity>,
        username: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        let candidates = self
            .posts
            .list_all_by_username(username, 1, USER_POST_FILTER_FETCH_LIMIT)
            .await?;
        self.filter_user_post_page(viewer, candidates.items, page, page_size)
            .await
    }

    pub async fn list_user_star_posts(
        &self,
        username: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        self.posts
            .list_stars_by_username(username, page, page_size)
            .await
    }

    pub async fn list_user_star_posts_for_viewer(
        &self,
        viewer: Option<&UserIdentity>,
        username: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        let candidates = self
            .posts
            .list_stars_by_username(username, 1, USER_POST_FILTER_FETCH_LIMIT)
            .await?;
        self.filter_user_post_page(viewer, candidates.items, page, page_size)
            .await
    }

    pub async fn list_user_highlight_posts(
        &self,
        username: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        self.posts
            .list_highlighted_by_username(username, page, page_size)
            .await
    }

    pub async fn list_user_highlight_posts_for_viewer(
        &self,
        viewer: Option<&UserIdentity>,
        username: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        let candidates = self
            .posts
            .list_highlighted_by_username(username, 1, USER_POST_FILTER_FETCH_LIMIT)
            .await?;
        self.filter_user_post_page(viewer, candidates.items, page, page_size)
            .await
    }

    pub async fn list_user_media_posts(
        &self,
        username: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        self.posts
            .list_media_by_username(username, page, page_size)
            .await
    }

    pub async fn list_user_media_posts_for_viewer(
        &self,
        viewer: Option<&UserIdentity>,
        username: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        let candidates = self
            .posts
            .list_media_by_username(username, 1, USER_POST_FILTER_FETCH_LIMIT)
            .await?;
        self.filter_user_post_page(viewer, candidates.items, page, page_size)
            .await
    }

    pub async fn list_user_commented_posts(
        &self,
        username: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        self.posts
            .list_commented_posts_by_username(username, page, page_size)
            .await
    }

    pub async fn list_user_commented_posts_for_viewer(
        &self,
        viewer: Option<&UserIdentity>,
        username: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        let candidates = self
            .posts
            .list_commented_posts_by_username(username, 1, USER_POST_FILTER_FETCH_LIMIT)
            .await?;
        self.filter_user_post_page(viewer, candidates.items, page, page_size)
            .await
    }

    pub async fn list_user_collections(
        &self,
        actor: &UserIdentity,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        self.list_user_collections_for_viewer(actor, actor, page, page_size)
            .await
    }

    pub async fn list_user_collections_for_viewer(
        &self,
        owner: &UserIdentity,
        viewer: &UserIdentity,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        let candidates = self
            .posts
            .list_collections_by_user_id(owner.id, 1, USER_POST_FILTER_FETCH_LIMIT)
            .await?;
        self.filter_user_post_page(Some(viewer), candidates.items, page, page_size)
            .await
    }

    pub async fn list_viewer_collections(
        &self,
        actor: &UserIdentity,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        self.posts
            .list_collections_by_user_id(actor.id, page, page_size)
            .await
    }

    pub async fn has_starred_post(
        &self,
        actor: &UserIdentity,
        post_id: i64,
    ) -> Result<bool, AppError> {
        self.get_post(post_id).await?;
        self.posts.has_star(post_id, actor.id).await
    }

    pub async fn toggle_post_star(
        &self,
        actor: &UserIdentity,
        post_id: i64,
    ) -> Result<bool, AppError> {
        self.get_post(post_id).await?;
        if self.posts.has_star(post_id, actor.id).await? {
            self.posts.delete_star(post_id, actor.id).await?;
            Ok(false)
        } else {
            self.posts.create_star(post_id, actor.id).await?;
            Ok(true)
        }
    }

    pub async fn has_collected_post(
        &self,
        actor: &UserIdentity,
        post_id: i64,
    ) -> Result<bool, AppError> {
        self.get_post(post_id).await?;
        self.posts.has_collection(post_id, actor.id).await
    }

    pub async fn toggle_post_collection(
        &self,
        actor: &UserIdentity,
        post_id: i64,
    ) -> Result<bool, AppError> {
        self.get_post(post_id).await?;
        if self.posts.has_collection(post_id, actor.id).await? {
            self.posts.delete_collection(post_id, actor.id).await?;
            Ok(false)
        } else {
            self.posts.create_collection(post_id, actor.id).await?;
            Ok(true)
        }
    }

    pub async fn create_comment(
        &self,
        actor: &UserIdentity,
        post_id: i64,
        content: &str,
    ) -> Result<CommentSummary, AppError> {
        let content = content.trim();
        if normalized_comment_has_content(content)? == 0 {
            return Err(AppError::Validation(
                "comment content cannot be empty".into(),
            ));
        }
        self.get_post(post_id).await?;
        self.comments.create(post_id, actor.id, content).await
    }

    pub async fn create_comment_with_contents(
        &self,
        actor: &UserIdentity,
        post_id: i64,
        contents: &[CreateContentInput],
    ) -> Result<CommentSummary, AppError> {
        if contents.is_empty() {
            return Err(AppError::Validation(
                "comment content cannot be empty".into(),
            ));
        }

        self.get_post(post_id).await?;
        let comment = self
            .comments
            .create(post_id, actor.id, &summary_text(contents))
            .await?;
        self.legacy_posts.ensure_comment_state(comment.id).await?;

        for item in contents {
            self.comments
                .create_content(
                    comment.id,
                    actor.id,
                    &item.content,
                    item.content_type,
                    item.sort,
                )
                .await?;
        }
        self.legacy_posts
            .set_comment_reaction(comment.id, false)
            .await?;

        Ok(comment)
    }

    pub async fn list_comments(
        &self,
        post_id: i64,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<CommentSummary>, AppError> {
        self.get_post(post_id).await?;
        self.comments.list_by_post(post_id, page, page_size).await
    }

    pub async fn list_legacy_comments(
        &self,
        post_id: i64,
        style: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<CommentSummary>, AppError> {
        self.get_post(post_id).await?;
        self.comments
            .list_by_post_with_style(post_id, style, page, page_size)
            .await
    }

    pub async fn list_comment_contents(
        &self,
        comment_ids: &[i64],
    ) -> Result<Vec<CommentContentItem>, AppError> {
        self.comments
            .list_contents_by_comment_ids(comment_ids)
            .await
    }

    pub async fn delete_comment(
        &self,
        actor: &UserIdentity,
        comment_id: i64,
    ) -> Result<(), AppError> {
        let existing = self
            .comments
            .find_by_id(comment_id)
            .await?
            .ok_or_else(|| AppError::NotFound("comment not found".into()))?;
        let current = self.get_current_user(actor).await?;
        if existing.user_id != actor.id && !current.is_admin {
            return Err(AppError::Unauthorized(
                "cannot delete another user's comment".into(),
            ));
        }
        self.comments.delete_by_id(comment_id).await
    }

    pub async fn list_post_reactions(
        &self,
        viewer: Option<&UserIdentity>,
        post_id: i64,
    ) -> Result<Vec<evt_domain::PostReactionSummary>, AppError> {
        self.get_post(post_id).await?;
        let rows = self.comments.list_post_reaction_comments(post_id).await?;
        Ok(group_post_reactions(rows, viewer.map(|item| item.id)))
    }

    pub async fn toggle_post_reaction(
        &self,
        actor: &UserIdentity,
        post_id: i64,
        emoji: &str,
    ) -> Result<TogglePostReactionResult, AppError> {
        let emoji = emoji.trim();
        if emoji.is_empty() {
            return Err(AppError::Validation("reaction emoji cannot be empty".into()));
        }
        self.get_post(post_id).await?;

        let existing_rows = self.comments.list_post_reaction_comments(post_id).await?;
        let existing = existing_rows.iter().find(|row| row.user_id == actor.id && row.emoji == emoji);

        let active = if let Some(existing) = existing {
            self.comments
                .delete_reaction_comment(existing.comment_id, actor.id)
                .await?;
            false
        } else {
            self.create_comment_with_contents(
                actor,
                post_id,
                &[CreateContentInput {
                    content: emoji.to_string(),
                    content_type: 2,
                    sort: 100,
                }],
            )
            .await?;
            let latest_comment_id = self
                .comments
                .list_post_reaction_comments(post_id)
                .await?
                .into_iter()
                .filter(|row| row.user_id == actor.id && row.emoji == emoji)
                .map(|row| row.comment_id)
                .max()
                .ok_or_else(|| AppError::Internal("created reaction comment cannot be loaded".into()))?;
            self.legacy_posts
                .set_comment_reaction(latest_comment_id, true)
                .await?;
            true
        };

        let reactions = self.list_post_reactions(Some(actor), post_id).await?;
        let post = self.get_post(post_id).await?;

        Ok(TogglePostReactionResult {
            active,
            reactions,
            comment_count: post.comments_count,
        })
    }

    async fn filter_user_post_page(
        &self,
        viewer: Option<&UserIdentity>,
        posts: Vec<PostSummary>,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        let filtered = self.filter_visible_user_posts(viewer, posts).await?;
        let total = filtered.len() as i64;
        let offset = ((page.saturating_sub(1)) * page_size) as usize;
        let items = filtered
            .into_iter()
            .skip(offset)
            .take(page_size as usize)
            .collect();

        Ok(PagedResponse {
            items,
            total,
            page,
            page_size,
        })
    }

    async fn filter_visible_user_posts(
        &self,
        viewer: Option<&UserIdentity>,
        posts: Vec<PostSummary>,
    ) -> Result<Vec<PostSummary>, AppError> {
        if posts.is_empty() {
            return Ok(posts);
        }

        let post_ids = posts.iter().map(|item| item.id).collect::<Vec<_>>();
        let post_states = self.legacy_post_states_by_ids(&post_ids).await?;

        let mut author_ids = posts.iter().map(|item| item.user_id).collect::<Vec<_>>();
        author_ids.sort_unstable();
        author_ids.dedup();

        let (following_status, friend_status) = match viewer {
            Some(viewer) if !author_ids.is_empty() => (
                self.batch_following_status(viewer.id, &author_ids).await?,
                self.batch_friend_status(viewer.id, &author_ids).await?,
            ),
            _ => (
                std::collections::HashMap::new(),
                std::collections::HashMap::new(),
            ),
        };

        let mut visible = Vec::with_capacity(posts.len());
        for post in posts {
            if self
                .can_view_post_summary(
                    viewer,
                    &post,
                    post_states.get(&post.id),
                    &following_status,
                    &friend_status,
                )
                .await?
            {
                visible.push(post);
            }
        }

        Ok(visible)
    }

    async fn can_view_post_summary(
        &self,
        viewer: Option<&UserIdentity>,
        post: &PostSummary,
        state: Option<&LegacyPostState>,
        following_status: &std::collections::HashMap<i64, bool>,
        friend_status: &std::collections::HashMap<i64, bool>,
    ) -> Result<bool, AppError> {
        if self
            .ensure_can_access_space_id(viewer, post.space_id)
            .await
            .is_err()
        {
            return Ok(false);
        }

        if viewer.is_some_and(|item| item.id == post.user_id) {
            return Ok(true);
        }

        let visibility = state.map(|item| item.visibility).unwrap_or(0);
        let is_following = viewer
            .map(|_| {
                following_status
                    .get(&post.user_id)
                    .copied()
                    .unwrap_or(false)
            })
            .unwrap_or(false);
        let is_friend = viewer
            .map(|_| friend_status.get(&post.user_id).copied().unwrap_or(false))
            .unwrap_or(false);

        Ok(match visibility {
            0 => true,
            1 => false,
            2 => is_friend,
            3 => is_following,
            _ => false,
        })
    }
}

fn normalize_tags(tags: &[String]) -> Vec<String> {
    let mut normalized = Vec::new();
    for tag in tags {
        let trimmed = tag.trim().trim_start_matches('#').to_string();
        if trimmed.is_empty() {
            continue;
        }
        if !normalized.iter().any(|item| item == &trimmed) {
            normalized.push(trimmed);
        }
    }
    normalized
}

fn normalized_post_length(content: &str) -> Result<usize, AppError> {
    if let Some(json) = content.strip_prefix(LEGACY_CONTENT_PREFIX) {
        let payload: serde_json::Value = serde_json::from_str(json)
            .map_err(|err| AppError::Validation(format!("invalid legacy post payload: {err}")))?;
        let contents = payload["contents"]
            .as_array()
            .ok_or_else(|| AppError::Validation("legacy post payload missing contents".into()))?;
        if contents.is_empty() {
            return Ok(0);
        }

        let length: usize = contents
            .iter()
            .filter_map(|item| item["content"].as_str())
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(|item: &str| item.len())
            .sum();
        return Ok(length.max(1));
    }

    Ok(content.len())
}

fn normalized_comment_has_content(content: &str) -> Result<usize, AppError> {
    if let Some(json) = content.strip_prefix(LEGACY_CONTENT_PREFIX) {
        let payload: serde_json::Value = serde_json::from_str(json).map_err(|err| {
            AppError::Validation(format!("invalid legacy comment payload: {err}"))
        })?;
        let contents = payload["contents"].as_array().ok_or_else(|| {
            AppError::Validation("legacy comment payload missing contents".into())
        })?;

        return Ok(contents.len());
    }

    Ok(usize::from(!content.is_empty()))
}

fn validate_contents(contents: &[CreateContentInput], max_text_len: usize) -> Result<(), AppError> {
    if contents.is_empty() {
        return Err(AppError::Validation("post content cannot be empty".into()));
    }

    let text_len = contents
        .iter()
        .filter(|item| matches!(item.content_type, 1 | 2 | 6))
        .map(|item| item.content.trim().len())
        .sum::<usize>();

    if text_len == 0 && contents.iter().all(|item| item.content.trim().is_empty()) {
        return Err(AppError::Validation("post content cannot be empty".into()));
    }

    if text_len > max_text_len {
        return Err(AppError::Validation(format!(
            "post content exceeds max length {}",
            max_text_len
        )));
    }

    Ok(())
}

fn validate_media_permissions(
    contents: &[CreateContentInput],
    site: &evt_domain::SiteProfile,
) -> Result<(), AppError> {
    if !site.allow_tweet_video
        && contents
            .iter()
            .any(|item| item.content_type == 4 && !item.content.trim().is_empty())
    {
        return Err(AppError::Validation("tweet video is disabled".into()));
    }

    Ok(())
}

fn summary_text(contents: &[CreateContentInput]) -> String {
    let text = contents
        .iter()
        .filter(|item| matches!(item.content_type, 1 | 2 | 6))
        .map(|item| item.content.trim())
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    if text.is_empty() {
        contents
            .iter()
            .map(|item| item.content.trim())
            .find(|item| !item.is_empty())
            .unwrap_or("")
            .to_string()
    } else {
        text
    }
}

#[cfg(test)]
mod tests {
    use evt_domain::{AppError, CreateContentInput, SiteProfile};

    use super::{
        LEGACY_CONTENT_PREFIX, normalize_tags, normalized_comment_has_content,
        normalized_post_length, validate_contents, validate_media_permissions,
    };

    #[test]
    fn normalize_tags_deduplicates_and_strips_prefix() {
        let tags = vec![
            " #rust ".to_string(),
            "#rust".to_string(),
            "evt".to_string(),
            "   ".to_string(),
        ];

        assert_eq!(
            normalize_tags(&tags),
            vec!["rust".to_string(), "evt".to_string()]
        );
    }

    #[test]
    fn normalized_post_length_supports_legacy_payloads() {
        let payload = format!(
            "{}{{\"contents\":[{{\"content\":\" hello \",\"type\":2}},{{\"content\":\"world\",\"type\":2}}]}}",
            LEGACY_CONTENT_PREFIX
        );

        assert_eq!(
            normalized_post_length(&payload).expect("legacy payload"),
            10
        );
        assert_eq!(
            normalized_post_length("plain text").expect("plain text"),
            10
        );
    }

    #[test]
    fn normalized_post_length_rejects_invalid_legacy_payload() {
        let payload = format!("{}not-json", LEGACY_CONTENT_PREFIX);
        let error = normalized_post_length(&payload).expect_err("invalid payload should fail");

        assert!(
            matches!(error, AppError::Validation(message) if message.contains("invalid legacy post payload"))
        );
    }

    #[test]
    fn normalized_comment_has_content_supports_legacy_payloads() {
        let payload = format!(
            "{}{{\"contents\":[{{\"content\":\"one\"}},{{\"content\":\"two\"}}]}}",
            LEGACY_CONTENT_PREFIX
        );

        assert_eq!(
            normalized_comment_has_content(&payload).expect("legacy comment payload"),
            2
        );
        assert_eq!(
            normalized_comment_has_content("hello").expect("plain comment"),
            1
        );
        assert_eq!(
            normalized_comment_has_content("").expect("empty comment"),
            0
        );
    }

    #[test]
    fn validate_contents_checks_presence_and_text_length() {
        let empty = validate_contents(&[], 20).expect_err("empty contents should fail");
        assert!(
            matches!(empty, AppError::Validation(message) if message.contains("post content cannot be empty"))
        );

        let long_text = vec![CreateContentInput {
            content: "x".repeat(21),
            content_type: 2,
            sort: 100,
        }];
        let long_error =
            validate_contents(&long_text, 20).expect_err("too long content should fail");
        assert!(
            matches!(long_error, AppError::Validation(message) if message.contains("post content exceeds max length"))
        );

        let media_only = vec![CreateContentInput {
            content: "https://cdn.example.com/demo.png".to_string(),
            content_type: 3,
            sort: 100,
        }];
        validate_contents(&media_only, 20).expect("media-only content remains allowed");
    }

    #[test]
    fn validate_media_permissions_blocks_video_when_disabled() {
        let contents = vec![CreateContentInput {
            content: "https://cdn.example.com/demo.mp4".to_string(),
            content_type: 4,
            sort: 100,
        }];
        let site = SiteProfile {
            default_space_slug: "public".into(),
            enable_spaces: true,
            use_friendship: true,
            enable_trends_bar: true,
            enable_wallet: false,
            allow_tweet_attachment: true,
            allow_tweet_attachment_price: false,
            allow_tweet_video: false,
            allow_user_register: true,
            allow_phone_bind: true,
            default_tweet_max_length: 2000,
            tweet_web_ellipsis_size: 400,
            tweet_mobile_ellipsis_size: 300,
            default_tweet_visibility: "public".into(),
            default_msg_loop_interval: 5000,
            copyright_top: String::new(),
            copyright_left: String::new(),
            copyright_left_link: String::new(),
            copyright_right: String::new(),
            copyright_right_link: String::new(),
        };

        let error =
            validate_media_permissions(&contents, &site).expect_err("video should be blocked");
        assert!(
            matches!(error, AppError::Validation(message) if message.contains("tweet video is disabled"))
        );
    }
}
