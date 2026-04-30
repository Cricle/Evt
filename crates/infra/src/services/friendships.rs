use paopao_domain::{AppError, PagedResponse, UserIdentity, UserPreview};

use crate::{
    AppContext,
    repository::friendships::{
        FRIEND_STATUS_AGREE, FRIEND_STATUS_DELETED, FRIEND_STATUS_REJECT, FRIEND_STATUS_REQUESTING,
    },
};

impl AppContext {
    pub async fn request_friend(
        &self,
        actor: &UserIdentity,
        friend_id: i64,
        greetings: &str,
    ) -> Result<(), AppError> {
        if actor.id == friend_id {
            return Err(AppError::Validation("cannot request yourself".into()));
        }
        let friend = self
            .users
            .find_by_id(friend_id)
            .await?
            .ok_or_else(|| AppError::NotFound("friend user not found".into()))?;
        self.profiles
            .ensure_defaults(friend.id, &friend.username)
            .await?;
        self.friendships.request(actor.id, friend_id).await?;
        self.messages
            .create_legacy(
                actor.id,
                friend_id,
                5,
                "请求添加好友，并附言:",
                greetings.trim(),
                0,
                0,
                FRIEND_STATUS_REQUESTING as i64,
            )
            .await?;
        Ok(())
    }

    pub async fn add_friend(&self, actor: &UserIdentity, friend_id: i64) -> Result<(), AppError> {
        self.friendships.approve(actor.id, friend_id).await?;
        self.sync_friend_request_status(friend_id, actor.id, FRIEND_STATUS_AGREE as i64)
            .await
    }

    pub async fn reject_friend(
        &self,
        actor: &UserIdentity,
        friend_id: i64,
    ) -> Result<(), AppError> {
        self.friendships.reject(actor.id, friend_id).await?;
        self.sync_friend_request_status(friend_id, actor.id, FRIEND_STATUS_REJECT as i64)
            .await
    }

    pub async fn delete_friend(
        &self,
        actor: &UserIdentity,
        friend_id: i64,
    ) -> Result<(), AppError> {
        self.friendships.delete_pair(actor.id, friend_id).await?;
        self.sync_friend_request_status(actor.id, friend_id, FRIEND_STATUS_DELETED as i64)
            .await
    }

    pub async fn is_friend(&self, user_id: i64, friend_id: i64) -> Result<bool, AppError> {
        self.friendships.is_friend(user_id, friend_id).await
    }

    pub async fn batch_friend_status(
        &self,
        user_id: i64,
        friend_ids: &[i64],
    ) -> Result<std::collections::HashMap<i64, bool>, AppError> {
        self.friendships
            .batch_friend_status(user_id, friend_ids)
            .await
    }

    pub async fn list_friend_contacts(
        &self,
        actor: &UserIdentity,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<UserPreview>, AppError> {
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let limit = page_size as i64;

        let total = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM friendships
            WHERE user_id = ? AND status = ?
            "#,
        )
        .bind(actor.id)
        .bind(FRIEND_STATUS_AGREE)
        .fetch_one(&self.pool)
        .await
        .map_err(crate::repository::map_db_error)?;

        let friend_ids = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT friend_id
            FROM friendships
            WHERE user_id = ? AND status = ?
            ORDER BY id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(actor.id)
        .bind(FRIEND_STATUS_AGREE)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(crate::repository::map_db_error)?;

        let mut items = Vec::with_capacity(friend_ids.len());
        for friend_id in friend_ids {
            if let Some(user) = self.users.find_preview_by_id(friend_id).await? {
                items.push(user);
            }
        }

        Ok(PagedResponse {
            items,
            total,
            page,
            page_size,
        })
    }

    async fn sync_friend_request_status(
        &self,
        sender_user_id: i64,
        receiver_user_id: i64,
        status: i64,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE messages
            SET reply_id = ?
            WHERE sender_user_id = ? AND receiver_user_id = ? AND type = 5 AND reply_id = ?
            "#,
        )
        .bind(status)
        .bind(sender_user_id)
        .bind(receiver_user_id)
        .bind(FRIEND_STATUS_REQUESTING as i64)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(crate::repository::map_db_error)
    }
}
