use paopao_domain::{
    AppError, FollowActionResult, PagedResponse, UserIdentity, UserPreview, UserSummary,
};

use crate::AppContext;

impl AppContext {
    pub async fn follow_user(
        &self,
        actor: &UserIdentity,
        target_username: &str,
    ) -> Result<FollowActionResult, AppError> {
        let target = self
            .users
            .find_summary_by_username(target_username)
            .await?
            .ok_or_else(|| AppError::NotFound("target user not found".into()))?;

        if actor.id == target.id {
            return Err(AppError::Validation("cannot follow yourself".into()));
        }

        self.follows.follow(actor.id, target.id).await?;

        Ok(FollowActionResult {
            following: true,
            user: target,
        })
    }

    pub async fn unfollow_user(
        &self,
        actor: &UserIdentity,
        target_username: &str,
    ) -> Result<FollowActionResult, AppError> {
        let target = self
            .users
            .find_summary_by_username(target_username)
            .await?
            .ok_or_else(|| AppError::NotFound("target user not found".into()))?;

        if actor.id == target.id {
            return Err(AppError::Validation("cannot unfollow yourself".into()));
        }

        self.follows.unfollow(actor.id, target.id).await?;

        Ok(FollowActionResult {
            following: false,
            user: target,
        })
    }

    pub async fn follow_user_by_id(
        &self,
        actor: &UserIdentity,
        target_user_id: i64,
    ) -> Result<FollowActionResult, AppError> {
        let target = self
            .users
            .find_summary_by_id(target_user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("target user not found".into()))?;

        if actor.id == target.id {
            return Err(AppError::Validation("cannot follow yourself".into()));
        }

        self.follows.follow(actor.id, target.id).await?;

        Ok(FollowActionResult {
            following: true,
            user: target,
        })
    }

    pub async fn unfollow_user_by_id(
        &self,
        actor: &UserIdentity,
        target_user_id: i64,
    ) -> Result<FollowActionResult, AppError> {
        let target = self
            .users
            .find_summary_by_id(target_user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("target user not found".into()))?;

        if actor.id == target.id {
            return Err(AppError::Validation("cannot unfollow yourself".into()));
        }

        self.follows.unfollow(actor.id, target.id).await?;

        Ok(FollowActionResult {
            following: false,
            user: target,
        })
    }

    pub async fn list_followers(
        &self,
        username: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<UserSummary>, AppError> {
        self.follows.list_followers(username, page, page_size).await
    }

    pub async fn list_followings(
        &self,
        username: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<UserSummary>, AppError> {
        self.follows
            .list_followings(username, page, page_size)
            .await
    }

    pub async fn list_follower_previews(
        &self,
        username: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<UserPreview>, AppError> {
        self.follows
            .list_follower_previews(username, page, page_size)
            .await
    }

    pub async fn list_following_previews(
        &self,
        username: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<UserPreview>, AppError> {
        self.follows
            .list_following_previews(username, page, page_size)
            .await
    }

    pub async fn is_following(&self, follower_id: i64, followee_id: i64) -> Result<bool, AppError> {
        self.follows.is_following(follower_id, followee_id).await
    }

    pub async fn batch_following_status(
        &self,
        follower_id: i64,
        followee_ids: &[i64],
    ) -> Result<std::collections::HashMap<i64, bool>, AppError> {
        self.follows
            .batch_following_status(follower_id, followee_ids)
            .await
    }
}
