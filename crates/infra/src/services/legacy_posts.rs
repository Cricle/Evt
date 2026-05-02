use std::collections::HashMap;

use evt_domain::{AppError, CreateContentInput, LegacyCommentState, LegacyPostState, UserIdentity};

use crate::AppContext;

impl AppContext {
    pub fn validate_legacy_visibility(&self, visibility: i32) -> Result<(), AppError> {
        match visibility {
            0..=3 => Ok(()),
            _ => Err(AppError::Validation("invalid legacy visibility".into())),
        }
    }

    pub async fn create_legacy_post(
        &self,
        actor: &UserIdentity,
        contents: &[CreateContentInput],
        tags: &[String],
        attachment_price: i64,
        visibility: i32,
    ) -> Result<evt_domain::PostSummary, AppError> {
        self.validate_legacy_visibility(visibility)?;
        self.create_legacy_post_in_space(actor, None, contents, tags, attachment_price, visibility)
            .await
    }

    pub async fn create_legacy_post_in_space(
        &self,
        actor: &UserIdentity,
        space_slug: Option<&str>,
        contents: &[CreateContentInput],
        tags: &[String],
        attachment_price: i64,
        visibility: i32,
    ) -> Result<evt_domain::PostSummary, AppError> {
        self.validate_legacy_visibility(visibility)?;
        let space_id = self.resolve_space(Some(actor), space_slug).await?.id;
        let post = self
            .create_post_with_contents_and_tags(actor, space_id, contents, tags)
            .await?;
        self.legacy_posts
            .ensure_post_state(post.id, attachment_price.max(0), visibility)
            .await?;
        Ok(post)
    }

    pub async fn legacy_post_states_by_ids(
        &self,
        post_ids: &[i64],
    ) -> Result<HashMap<i64, LegacyPostState>, AppError> {
        self.legacy_posts.post_states_by_ids(post_ids).await
    }

    pub async fn toggle_post_lock(
        &self,
        actor: &UserIdentity,
        post_id: i64,
    ) -> Result<bool, AppError> {
        let post = self.get_post(post_id).await?;
        let current = self.get_current_user(actor).await?;
        if !current.is_admin {
            self.ensure_can_access_space_id(Some(actor), post.space_id)
                .await?;
        }
        self.ensure_legacy_post_state(post_id).await?;
        self.legacy_posts.toggle_post_flag(post_id, "is_lock").await
    }

    pub async fn toggle_post_top(
        &self,
        actor: &UserIdentity,
        post_id: i64,
    ) -> Result<bool, AppError> {
        let post = self.get_post(post_id).await?;
        let current = self.get_current_user(actor).await?;
        if !current.is_admin {
            self.ensure_can_access_space_id(Some(actor), post.space_id)
                .await?;
        }
        self.ensure_legacy_post_state(post_id).await?;
        self.legacy_posts.toggle_post_flag(post_id, "is_top").await
    }

    pub async fn toggle_post_essence(
        &self,
        actor: &UserIdentity,
        post_id: i64,
    ) -> Result<bool, AppError> {
        let post = self.get_post(post_id).await?;
        let current = self.get_current_user(actor).await?;
        if !current.is_admin {
            self.ensure_can_access_space_id(Some(actor), post.space_id)
                .await?;
        }
        self.ensure_legacy_post_state(post_id).await?;
        self.legacy_posts
            .toggle_post_flag(post_id, "is_essence")
            .await
    }

    pub async fn set_post_visibility(
        &self,
        actor: &UserIdentity,
        post_id: i64,
        visibility: i32,
    ) -> Result<i32, AppError> {
        self.validate_legacy_visibility(visibility)?;
        let post = self.get_post(post_id).await?;
        let current = self.get_current_user(actor).await?;
        if !current.is_admin {
            self.ensure_can_access_space_id(Some(actor), post.space_id)
                .await?;
        }
        self.ensure_legacy_post_state(post_id).await?;
        self.legacy_posts
            .set_post_visibility(post_id, visibility)
            .await
    }

    pub async fn create_legacy_comment(
        &self,
        actor: &UserIdentity,
        post_id: i64,
        contents: &[CreateContentInput],
    ) -> Result<evt_domain::CommentSummary, AppError> {
        let comment = self
            .create_comment_with_contents(actor, post_id, contents)
            .await?;
        self.legacy_posts.ensure_comment_state(comment.id).await?;

        let post = self.get_post(post_id).await?;
        if post.user_id != actor.id {
            let _ = self
                .send_legacy_message(
                    actor.id,
                    post.user_id,
                    2,
                    "在 Evt 中评论了你",
                    "",
                    post.id,
                    comment.id,
                    0,
                )
                .await;
        }

        Ok(comment)
    }

    pub async fn legacy_comment_states_by_ids(
        &self,
        comment_ids: &[i64],
    ) -> Result<HashMap<i64, LegacyCommentState>, AppError> {
        self.legacy_posts.comment_states_by_ids(comment_ids).await
    }

    pub async fn toggle_comment_essence(
        &self,
        actor: &UserIdentity,
        comment_id: i64,
    ) -> Result<bool, AppError> {
        let comment = self
            .comments
            .find_by_id(comment_id)
            .await?
            .ok_or_else(|| AppError::NotFound("comment not found".into()))?;
        let post = self.get_post(comment.post_id).await?;
        self.ensure_can_access_space_id(Some(actor), post.space_id)
            .await?;
        self.legacy_posts.ensure_comment_state(comment_id).await?;
        self.legacy_posts.toggle_comment_essence(comment_id).await
    }

    pub async fn toggle_comment_thumb(
        &self,
        actor: &UserIdentity,
        post_id: i64,
        comment_id: i64,
        thumbs_up: bool,
    ) -> Result<(), AppError> {
        let comment = self
            .comments
            .find_by_id(comment_id)
            .await?
            .ok_or_else(|| AppError::NotFound("comment not found".into()))?;
        if comment.post_id != post_id {
            return Err(AppError::Validation(
                "comment does not belong to the post".into(),
            ));
        }
        let post = self.get_post(post_id).await?;
        self.ensure_can_access_space_id(Some(actor), post.space_id)
            .await?;
        self.legacy_posts
            .toggle_reaction(actor.id, post_id, comment_id, thumbs_up)
            .await
    }

    pub async fn comment_thumb_counts(
        &self,
        comment_ids: &[i64],
    ) -> Result<HashMap<i64, i64>, AppError> {
        self.legacy_posts
            .reaction_counts_by_comments(comment_ids)
            .await
    }

    pub async fn reaction_status_map(
        &self,
        user_id: i64,
        comment_ids: &[i64],
    ) -> Result<HashMap<(i32, i64), (bool, bool)>, AppError> {
        self.legacy_posts
            .reaction_status_map(user_id, comment_ids)
            .await
    }

    async fn ensure_legacy_post_state(&self, post_id: i64) -> Result<(), AppError> {
        let state = self
            .legacy_posts
            .post_states_by_ids(&[post_id])
            .await?
            .remove(&post_id)
            .unwrap_or_default();
        self.legacy_posts
            .ensure_post_state(post_id, state.attachment_price, state.visibility)
            .await
    }
}
