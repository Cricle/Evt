use paopao_domain::{
    AppError, CommentContentItem, CommentSummary, CreateContentInput, PagedResponse,
    PostContentItem, PostSummary, UserIdentity,
};

use crate::AppContext;

const LEGACY_CONTENT_PREFIX: &str = "__PAOPAO_LEGACY_CONTENT__:";

impl AppContext {
    pub async fn create_post_with_contents(
        &self,
        actor: &UserIdentity,
        contents: &[CreateContentInput],
    ) -> Result<PostSummary, AppError> {
        self.create_post_with_contents_and_tags(actor, contents, &[])
            .await
    }

    pub async fn create_post_with_contents_and_tags(
        &self,
        actor: &UserIdentity,
        contents: &[CreateContentInput],
        tags: &[String],
    ) -> Result<PostSummary, AppError> {
        let site = self.site_profile_snapshot();
        validate_contents(contents, site.default_tweet_max_length as usize)?;
        let summary = summary_text(contents);
        let normalized_tags = normalize_tags(tags);
        let tags_value = normalized_tags.join(",");
        let post = self.posts.create(actor.id, &summary, &tags_value).await?;

        for item in contents {
            self.posts
                .create_content(
                    post.id,
                    actor.id,
                    &item.content,
                    item.content_type,
                    item.sort,
                )
                .await?;
        }

        if !normalized_tags.is_empty() {
            self.tags.record_tags(actor.id, &normalized_tags).await?;
        }

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

        self.posts.create(actor.id, content, "").await
    }

    pub async fn list_posts(
        &self,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        self.posts.list(page, page_size).await
    }

    pub async fn list_hot_posts(
        &self,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        self.posts.list_hot(page, page_size).await
    }

    pub async fn search_posts(
        &self,
        query: &str,
        query_type: Option<&str>,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        self.posts.search(query, query_type, page, page_size).await
    }

    pub async fn list_feed(
        &self,
        actor: &UserIdentity,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        self.posts.list_feed(actor.id, page, page_size).await
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

        self.posts
            .update_content(post_id, content)
            .await?
            .ok_or_else(|| AppError::Internal("updated post cannot be loaded".into()))
    }

    pub async fn delete_post(&self, actor: &UserIdentity, post_id: i64) -> Result<(), AppError> {
        let existing = self.get_post(post_id).await?;
        let current = self.get_current_user(actor).await?;
        if existing.user_id != actor.id && !current.is_admin {
            return Err(AppError::Unauthorized(
                "cannot delete another user's post".into(),
            ));
        }
        self.posts.delete_by_id(post_id).await
    }

    pub async fn list_user_posts(
        &self,
        username: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        self.posts.list_by_username(username, page, page_size).await
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

    pub async fn list_user_collections(
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
