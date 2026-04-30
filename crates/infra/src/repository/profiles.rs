use paopao_domain::{AppError, UserMeta};
use sqlx::{FromRow, MySqlPool};

use super::map_db_error;

#[derive(Clone)]
pub struct UserProfileRepository {
    pool: MySqlPool,
}

impl UserProfileRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn ensure_defaults(
        &self,
        user_id: i64,
        username: &str,
    ) -> Result<UserMeta, AppError> {
        sqlx::query(
            r#"
            INSERT INTO user_profiles (user_id, nickname)
            VALUES (?, ?)
            ON DUPLICATE KEY UPDATE user_id = user_id
            "#,
        )
        .bind(user_id)
        .bind(username)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        self.find_by_user_id(user_id)
            .await?
            .ok_or_else(|| AppError::Internal("user profile cannot be loaded".into()))
    }

    pub async fn find_by_user_id(&self, user_id: i64) -> Result<Option<UserMeta>, AppError> {
        sqlx::query_as::<_, UserMetaRow>(
            r#"
            SELECT user_id, nickname, avatar, activation_code, is_admin, balance
            FROM user_profiles
            WHERE user_id = ?
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Into::into))
        .map_err(map_db_error)
    }

    pub async fn update_nickname(&self, user_id: i64, nickname: &str) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE user_profiles
            SET nickname = ?
            WHERE user_id = ?
            "#,
        )
        .bind(nickname)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn update_avatar(&self, user_id: i64, avatar: &str) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE user_profiles
            SET avatar = ?
            WHERE user_id = ?
            "#,
        )
        .bind(avatar)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn update_activation(
        &self,
        user_id: i64,
        activation_code: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE user_profiles
            SET activation_code = ?
            WHERE user_id = ?
            "#,
        )
        .bind(activation_code)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn update_balance(&self, user_id: i64, balance: i64) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE user_profiles
            SET balance = ?
            WHERE user_id = ?
            "#,
        )
        .bind(balance)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn update_admin(&self, user_id: i64, is_admin: bool) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE user_profiles
            SET is_admin = ?
            WHERE user_id = ?
            "#,
        )
        .bind(is_admin)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }
}

#[derive(Debug, FromRow)]
struct UserMetaRow {
    user_id: i64,
    nickname: String,
    avatar: String,
    activation_code: String,
    is_admin: bool,
    balance: i64,
}

impl From<UserMetaRow> for UserMeta {
    fn from(row: UserMetaRow) -> Self {
        Self {
            user_id: row.user_id,
            nickname: row.nickname,
            avatar: row.avatar,
            activation_code: row.activation_code,
            is_admin: row.is_admin,
            balance: row.balance,
        }
    }
}
