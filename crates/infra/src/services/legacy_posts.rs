use std::collections::HashMap;

use paopao_domain::{
    AppError, CommentReplySummary, CreateContentInput, LegacyCommentState, LegacyPostState,
    UserIdentity,
};

use crate::{
    AppContext,
    repository::legacy_posts::{REACTION_TARGET_COMMENT, REACTION_TARGET_REPLY},
};

impl AppContext {
    pub async fn create_legacy_post(
        &self,
        actor: &UserIdentity,
        contents: &[CreateContentInput],
        tags: &[String],
        attachment_price: i64,
        visibility: i32,
    ) -> Result<paopao_domain::PostSummary, AppError> {
        let post = self
            .create_post_with_contents_and_tags(actor, contents, tags)
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

    pub async fn toggle_post_lock(&self, post_id: i64) -> Result<bool, AppError> {
        self.ensure_legacy_post_state(post_id).await?;
        self.legacy_posts.toggle_post_flag(post_id, "is_lock").await
    }

    pub async fn toggle_post_top(&self, post_id: i64) -> Result<bool, AppError> {
        self.ensure_legacy_post_state(post_id).await?;
        self.legacy_posts.toggle_post_flag(post_id, "is_top").await
    }

    pub async fn toggle_post_essence(&self, post_id: i64) -> Result<bool, AppError> {
        self.ensure_legacy_post_state(post_id).await?;
        self.legacy_posts
            .toggle_post_flag(post_id, "is_essence")
            .await
    }

    pub async fn set_post_visibility(
        &self,
        post_id: i64,
        visibility: i32,
    ) -> Result<i32, AppError> {
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
    ) -> Result<paopao_domain::CommentSummary, AppError> {
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
                    "在泡泡中评论了你",
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

    pub async fn toggle_comment_essence(&self, comment_id: i64) -> Result<bool, AppError> {
        self.legacy_posts.ensure_comment_state(comment_id).await?;
        self.legacy_posts.toggle_comment_essence(comment_id).await
    }

    pub async fn create_comment_reply(
        &self,
        actor: &UserIdentity,
        comment_id: i64,
        at_user_id: i64,
        content: &str,
    ) -> Result<CommentReplySummary, AppError> {
        let content = content.trim();
        if content.is_empty() {
            return Err(AppError::Validation("reply content cannot be empty".into()));
        }
        let comment = self
            .comments
            .find_by_id(comment_id)
            .await?
            .ok_or_else(|| AppError::NotFound("comment not found".into()))?;
        let reply = self
            .legacy_posts
            .create_reply(comment_id, actor.id, at_user_id, content)
            .await?;
        let post = self.get_post(comment.post_id).await?;

        if comment.user_id != actor.id {
            let _ = self
                .send_legacy_message(
                    actor.id,
                    comment.user_id,
                    3,
                    "在泡泡评论下回复了你",
                    "",
                    post.id,
                    comment.id,
                    reply.id,
                )
                .await;
        }
        if post.user_id != actor.id && post.user_id != comment.user_id {
            let _ = self
                .send_legacy_message(
                    actor.id,
                    post.user_id,
                    3,
                    "在泡泡评论下发布了新回复",
                    "",
                    post.id,
                    comment.id,
                    reply.id,
                )
                .await;
        }
        if at_user_id > 0
            && at_user_id != actor.id
            && at_user_id != comment.user_id
            && at_user_id != post.user_id
        {
            let _ = self
                .send_legacy_message(
                    actor.id,
                    at_user_id,
                    3,
                    "在泡泡评论的回复中@了你",
                    "",
                    post.id,
                    comment.id,
                    reply.id,
                )
                .await;
        }

        Ok(reply)
    }

    pub async fn delete_comment_reply(
        &self,
        actor: &UserIdentity,
        reply_id: i64,
    ) -> Result<(), AppError> {
        let reply = self
            .legacy_posts
            .reply_by_id(reply_id)
            .await?
            .ok_or_else(|| AppError::NotFound("reply not found".into()))?;
        let current = self.get_current_user(actor).await?;
        if reply.user_id != actor.id && !current.is_admin {
            return Err(AppError::Unauthorized(
                "cannot delete another user's reply".into(),
            ));
        }
        self.legacy_posts.delete_reply(reply_id).await
    }

    pub async fn list_comment_replies(
        &self,
        comment_ids: &[i64],
    ) -> Result<Vec<CommentReplySummary>, AppError> {
        self.legacy_posts.replies_by_comment_ids(comment_ids).await
    }

    pub async fn toggle_comment_thumb(
        &self,
        actor: &UserIdentity,
        post_id: i64,
        comment_id: i64,
        thumbs_up: bool,
    ) -> Result<(), AppError> {
        self.legacy_posts
            .toggle_reaction(
                actor.id,
                post_id,
                comment_id,
                0,
                REACTION_TARGET_COMMENT,
                thumbs_up,
            )
            .await
    }

    pub async fn toggle_reply_thumb(
        &self,
        actor: &UserIdentity,
        post_id: i64,
        comment_id: i64,
        reply_id: i64,
        thumbs_up: bool,
    ) -> Result<(), AppError> {
        self.legacy_posts
            .toggle_reaction(
                actor.id,
                post_id,
                comment_id,
                reply_id,
                REACTION_TARGET_REPLY,
                thumbs_up,
            )
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

    pub async fn reply_thumb_counts(
        &self,
        reply_ids: &[i64],
    ) -> Result<HashMap<i64, i64>, AppError> {
        self.legacy_posts
            .reaction_counts_by_replies(reply_ids)
            .await
    }

    pub async fn reaction_status_map(
        &self,
        user_id: i64,
        comment_ids: &[i64],
        reply_ids: &[i64],
    ) -> Result<HashMap<(i32, i64), (bool, bool)>, AppError> {
        self.legacy_posts
            .reaction_status_map(user_id, comment_ids, reply_ids)
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
