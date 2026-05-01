use evt_domain::{AppError, LoginResult, RegisterResult, UserIdentity, UserStatus};

use crate::AppContext;

impl AppContext {
    pub async fn authenticate_token(&self, token: &str) -> Result<UserIdentity, AppError> {
        let identity = self.jwt.verify(token)?;
        let user = self
            .users
            .find_by_id(identity.id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("user no longer exists".into()))?;

        if user.status != UserStatus::Active {
            return Err(AppError::Unauthorized("user account is disabled".into()));
        }

        Ok(UserIdentity {
            id: user.id,
            username: user.username,
        })
    }

    pub async fn register(
        &self,
        username: &str,
        password: &str,
    ) -> Result<RegisterResult, AppError> {
        if !self.site_profile_snapshot().allow_user_register {
            return Err(AppError::Validation("user registration is disabled".into()));
        }
        if username.trim().len() < 3 || password.len() < 6 {
            return Err(AppError::Validation(
                "username or password does not satisfy minimum length".into(),
            ));
        }

        if self.users.find_by_username(username).await?.is_some() {
            return Err(AppError::Conflict("username already exists".into()));
        }

        let user_count_before = self.users.count_all().await?;
        let password_hash = self.password.hash(password)?;
        let user = self
            .users
            .create_local_user(username, &password_hash)
            .await?;
        self.profiles
            .ensure_defaults(user.id, &user.username)
            .await?;
        if user_count_before == 0 {
            self.profiles.update_admin(user.id, true).await?;
            let default_space_slug = self.site_profile_snapshot().default_space_slug;
            self.spaces
                .ensure_default_space(&default_space_slug, user.id)
                .await?;
        }

        Ok(RegisterResult {
            id: user.id,
            username: user.username,
        })
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<LoginResult, AppError> {
        let user = self
            .users
            .find_by_username(username)
            .await?
            .ok_or_else(|| AppError::Unauthorized("invalid username or password".into()))?;

        if user.status != UserStatus::Active {
            return Err(AppError::Unauthorized("user account is disabled".into()));
        }

        let password_hash = user.password_hash.as_deref().ok_or_else(|| {
            AppError::Unauthorized("password login is not enabled for this account".into())
        })?;

        self.password.verify(password, password_hash)?;

        Ok(LoginResult {
            token: self.jwt.issue(UserIdentity {
                id: user.id,
                username: user.username,
            })?,
        })
    }

    pub async fn pre_login_by_phone(&self, phone_number: &str) -> Result<(), AppError> {
        if phone_number.trim().len() < 4 {
            return Err(AppError::Validation("phone number is too short".into()));
        }
        Ok(())
    }

    pub async fn login_by_phone(&self, phone_number: &str) -> Result<LoginResult, AppError> {
        self.pre_login_by_phone(phone_number).await?;

        let user = self.users.find_or_create_mobile_user(phone_number).await?;
        self.profiles
            .ensure_defaults(user.id, &user.username)
            .await?;
        if self.users.count_all().await? == 1 {
            self.profiles.update_admin(user.id, true).await?;
            let default_space_slug = self.site_profile_snapshot().default_space_slug;
            self.spaces
                .ensure_default_space(&default_space_slug, user.id)
                .await?;
        }

        Ok(LoginResult {
            token: self.jwt.issue(UserIdentity {
                id: user.id,
                username: user.username,
            })?,
        })
    }
}
