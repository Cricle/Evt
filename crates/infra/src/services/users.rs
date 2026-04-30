use std::collections::HashMap;

use paopao_domain::{
    AppError, CommentSummary, CurrentUser, UserIdentity, UserPreview, UserProfile,
};

use crate::AppContext;

impl AppContext {
    pub async fn get_user_preview_by_id(&self, user_id: i64) -> Result<UserPreview, AppError> {
        self.users
            .find_preview_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".into()))
    }

    pub async fn batch_user_previews_by_ids(
        &self,
        user_ids: &[i64],
    ) -> Result<HashMap<i64, UserPreview>, AppError> {
        let previews = self.users.find_previews_by_ids(user_ids).await?;
        Ok(previews.into_iter().map(|item| (item.id, item)).collect())
    }

    pub async fn get_user_profile(&self, username: &str) -> Result<UserProfile, AppError> {
        self.users
            .find_profile_by_username(username)
            .await?
            .ok_or_else(|| AppError::NotFound("user profile not found".into()))
    }

    pub async fn get_current_user(&self, actor: &UserIdentity) -> Result<CurrentUser, AppError> {
        self.profiles
            .ensure_defaults(actor.id, &actor.username)
            .await?;
        self.users
            .find_current_user(actor.id)
            .await?
            .ok_or_else(|| AppError::NotFound("current user not found".into()))
    }

    pub async fn change_nickname(
        &self,
        actor: &UserIdentity,
        nickname: &str,
    ) -> Result<CurrentUser, AppError> {
        let nickname = nickname.trim();
        if nickname.chars().count() < 2 || nickname.chars().count() > 16 {
            return Err(AppError::Validation(
                "nickname length must be between 2 and 16".into(),
            ));
        }
        self.profiles
            .ensure_defaults(actor.id, &actor.username)
            .await?;
        self.profiles.update_nickname(actor.id, nickname).await?;
        self.users
            .find_current_user(actor.id)
            .await?
            .ok_or_else(|| AppError::Internal("updated user cannot be loaded".into()))
    }

    pub async fn change_password(
        &self,
        actor: &UserIdentity,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), AppError> {
        if new_password.len() < 6 {
            return Err(AppError::Validation(
                "new password must be at least 6 characters".into(),
            ));
        }
        let user = self
            .users
            .find_by_id(actor.id)
            .await?
            .ok_or_else(|| AppError::NotFound("current user not found".into()))?;

        let password_hash = user.password_hash.as_deref().ok_or_else(|| {
            AppError::Unauthorized("password login is not enabled for this account".into())
        })?;
        self.password.verify(old_password, password_hash)?;
        let new_hash = self.password.hash(new_password)?;
        self.users.update_password_hash(actor.id, &new_hash).await
    }

    pub async fn suggest_usernames(
        &self,
        keyword: &str,
        limit: u64,
    ) -> Result<Vec<String>, AppError> {
        let keyword = keyword.trim();
        if keyword.is_empty() {
            return Ok(Vec::new());
        }

        self.users.search_usernames(keyword, limit).await
    }

    pub async fn update_avatar(&self, actor: &UserIdentity, avatar: &str) -> Result<(), AppError> {
        self.profiles
            .ensure_defaults(actor.id, &actor.username)
            .await?;
        self.profiles.update_avatar(actor.id, avatar.trim()).await
    }

    pub async fn bind_phone(
        &self,
        actor: &UserIdentity,
        phone: &str,
    ) -> Result<CurrentUser, AppError> {
        self.users
            .update_phone_number(actor.id, phone.trim())
            .await?;
        self.get_current_user(actor).await
    }

    pub async fn activate_user(
        &self,
        actor: &UserIdentity,
        activation_code: &str,
    ) -> Result<CurrentUser, AppError> {
        self.profiles
            .ensure_defaults(actor.id, &actor.username)
            .await?;
        self.profiles
            .update_activation(actor.id, activation_code.trim())
            .await?;
        self.get_current_user(actor).await
    }

    pub async fn update_user_status(&self, user_id: i64, active: bool) -> Result<(), AppError> {
        let status = if active { "active" } else { "disabled" };
        self.users.update_status(user_id, status).await
    }

    pub async fn update_user_admin(&self, user_id: i64, is_admin: bool) -> Result<(), AppError> {
        let user = self
            .users
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".into()))?;
        self.profiles
            .ensure_defaults(user.id, &user.username)
            .await?;
        self.profiles.update_admin(user.id, is_admin).await
    }

    pub async fn get_comment(&self, comment_id: i64) -> Result<CommentSummary, AppError> {
        self.comments
            .find_by_id(comment_id)
            .await?
            .ok_or_else(|| AppError::NotFound("comment not found".into()))
    }
}
