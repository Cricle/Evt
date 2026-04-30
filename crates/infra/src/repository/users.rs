use chrono::{DateTime, Utc};
use evt_domain::{AppError, CurrentUser, User, UserPreview, UserProfile, UserStatus, UserSummary};
use sqlx::{FromRow, MySql, MySqlPool, QueryBuilder};
use uuid::Uuid;

use super::map_db_error;

#[derive(Clone)]
pub struct UserRepository {
    pool: MySqlPool,
}

impl UserRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<User>, AppError> {
        sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, username, phone_number, password_hash, status, created_at, updated_at
            FROM users
            WHERE id = ?
            LIMIT 1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Into::into))
        .map_err(map_db_error)
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>, AppError> {
        sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, username, phone_number, password_hash, status, created_at, updated_at
            FROM users
            WHERE username = ?
            LIMIT 1
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Into::into))
        .map_err(map_db_error)
    }

    pub async fn find_current_user(&self, id: i64) -> Result<Option<CurrentUser>, AppError> {
        sqlx::query_as::<_, CurrentUserRow>(
            r#"
            SELECT
              u.id,
              u.username,
              COALESCE(up.nickname, u.username) AS nickname,
              COALESCE(up.avatar, '') AS avatar,
              u.phone_number,
              COALESCE(up.activation_code, '') AS activation_code,
              COALESCE(up.balance, 0) AS balance,
              COALESCE(up.is_admin, FALSE) AS is_admin,
              u.status,
              u.created_at,
              u.updated_at
            FROM users u
            LEFT JOIN user_profiles up ON up.user_id = u.id
            WHERE u.id = ?
            LIMIT 1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Into::into))
        .map_err(map_db_error)
    }

    pub async fn find_profile_by_username(
        &self,
        username: &str,
    ) -> Result<Option<UserProfile>, AppError> {
        sqlx::query_as::<_, UserProfileRow>(
            r#"
            SELECT
              u.id,
              u.username,
              COALESCE(up.nickname, u.username) AS nickname,
              COALESCE(up.avatar, '') AS avatar,
              COALESCE(up.is_admin, FALSE) AS is_admin,
              u.phone_number,
              COALESCE(up.activation_code, '') AS activation_code,
              COALESCE(up.balance, 0) AS balance,
              u.status,
              u.created_at,
              (SELECT COUNT(*) FROM posts p WHERE p.user_id = u.id) AS posts_count,
              (SELECT COUNT(*) FROM comments c WHERE c.user_id = u.id) AS comments_count,
              (SELECT COUNT(*) FROM follows f WHERE f.follower_id = u.id) AS followings_count,
              (SELECT COUNT(*) FROM follows f WHERE f.followee_id = u.id) AS followers_count
            FROM users u
            LEFT JOIN user_profiles up ON up.user_id = u.id
            WHERE u.username = ?
            LIMIT 1
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Into::into))
        .map_err(map_db_error)
    }

    pub async fn find_summary_by_username(
        &self,
        username: &str,
    ) -> Result<Option<UserSummary>, AppError> {
        sqlx::query_as::<_, UserSummaryRow>(
            r#"
            SELECT id, username, status
            FROM users
            WHERE username = ?
            LIMIT 1
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Into::into))
        .map_err(map_db_error)
    }

    pub async fn find_summary_by_id(&self, id: i64) -> Result<Option<UserSummary>, AppError> {
        sqlx::query_as::<_, UserSummaryRow>(
            r#"
            SELECT id, username, status
            FROM users
            WHERE id = ?
            LIMIT 1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Into::into))
        .map_err(map_db_error)
    }

    pub async fn find_preview_by_id(&self, id: i64) -> Result<Option<UserPreview>, AppError> {
        sqlx::query_as::<_, UserPreviewRow>(
            r#"
            SELECT
              u.id,
              u.username,
              COALESCE(up.nickname, u.username) AS nickname,
              COALESCE(up.avatar, '') AS avatar,
              u.created_at
            FROM users u
            LEFT JOIN user_profiles up ON up.user_id = u.id
            WHERE u.id = ?
            LIMIT 1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Into::into))
        .map_err(map_db_error)
    }

    pub async fn find_previews_by_ids(&self, ids: &[i64]) -> Result<Vec<UserPreview>, AppError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut builder: QueryBuilder<MySql> = QueryBuilder::new(
            r#"
            SELECT
              u.id,
              u.username,
              COALESCE(up.nickname, u.username) AS nickname,
              COALESCE(up.avatar, '') AS avatar,
              u.created_at
            FROM users u
            LEFT JOIN user_profiles up ON up.user_id = u.id
            WHERE u.id IN (
            "#,
        );
        let mut separated = builder.separated(", ");
        for id in ids {
            separated.push_bind(id);
        }
        separated.push_unseparated(")");

        builder
            .build_query_as::<UserPreviewRow>()
            .fetch_all(&self.pool)
            .await
            .map(|rows| rows.into_iter().map(Into::into).collect())
            .map_err(map_db_error)
    }

    pub async fn search_usernames(
        &self,
        keyword: &str,
        limit: u64,
    ) -> Result<Vec<String>, AppError> {
        let pattern = format!("%{}%", keyword.trim());
        sqlx::query_scalar::<_, String>(
            r#"
            SELECT username
            FROM users
            WHERE username LIKE ?
            ORDER BY username ASC
            LIMIT ?
            "#,
        )
        .bind(pattern)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)
    }

    pub async fn find_or_create_mobile_user(&self, phone_number: &str) -> Result<User, AppError> {
        let username = format!("mobile_{}", Uuid::new_v4().simple());
        self.create_mobile_user(&username, phone_number).await
    }

    pub async fn create_local_user(
        &self,
        username: &str,
        password_hash: &str,
    ) -> Result<User, AppError> {
        sqlx::query(
            r#"
            INSERT INTO users (username, password_hash, status)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(username)
        .bind(password_hash)
        .bind("active")
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        self.find_by_username(username)
            .await?
            .ok_or_else(|| AppError::Internal("created user cannot be loaded".into()))
    }

    async fn create_mobile_user(
        &self,
        username: &str,
        phone_number: &str,
    ) -> Result<User, AppError> {
        let result = sqlx::query(
            r#"
            INSERT INTO users (username, phone_number, status)
            VALUES (?, ?, ?)
            ON DUPLICATE KEY UPDATE
              id = LAST_INSERT_ID(id)
            "#,
        )
        .bind(username)
        .bind(phone_number)
        .bind("active")
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        self.find_by_id(result.last_insert_id() as i64)
            .await?
            .ok_or_else(|| AppError::Internal("mobile user cannot be loaded".into()))
    }
}
impl UserRepository {
    pub async fn update_password_hash(
        &self,
        user_id: i64,
        password_hash: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE users
            SET password_hash = ?
            WHERE id = ?
            "#,
        )
        .bind(password_hash)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn update_phone_number(
        &self,
        user_id: i64,
        phone_number: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE users
            SET phone_number = ?
            WHERE id = ?
            "#,
        )
        .bind(phone_number)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn update_status(&self, user_id: i64, status: &str) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE users
            SET status = ?
            WHERE id = ?
            "#,
        )
        .bind(status)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }
}

#[derive(Debug, FromRow)]
struct UserRow {
    id: i64,
    username: String,
    phone_number: Option<String>,
    password_hash: Option<String>,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct UserProfileRow {
    id: i64,
    username: String,
    nickname: String,
    avatar: String,
    is_admin: bool,
    phone_number: Option<String>,
    activation_code: String,
    balance: i64,
    status: String,
    created_at: DateTime<Utc>,
    posts_count: i64,
    comments_count: i64,
    followings_count: i64,
    followers_count: i64,
}

#[derive(Debug, FromRow)]
struct CurrentUserRow {
    id: i64,
    username: String,
    nickname: String,
    avatar: String,
    phone_number: Option<String>,
    activation_code: String,
    balance: i64,
    is_admin: bool,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct UserSummaryRow {
    id: i64,
    username: String,
    status: String,
}

#[derive(Debug, FromRow)]
struct UserPreviewRow {
    id: i64,
    username: String,
    nickname: String,
    avatar: String,
    created_at: DateTime<Utc>,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        let status = match row.status.as_str() {
            "disabled" => UserStatus::Disabled,
            _ => UserStatus::Active,
        };

        User {
            id: row.id,
            username: row.username,
            phone_number: row.phone_number,
            password_hash: row.password_hash,
            status,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<UserProfileRow> for UserProfile {
    fn from(row: UserProfileRow) -> Self {
        Self {
            id: row.id,
            username: row.username,
            nickname: row.nickname,
            avatar: row.avatar,
            is_admin: row.is_admin,
            is_friend: false,
            is_following: false,
            phone_number: row.phone_number,
            activation_code: row.activation_code,
            balance: row.balance,
            status: row.status,
            created_at: row.created_at,
            posts_count: row.posts_count,
            comments_count: row.comments_count,
            followings_count: row.followings_count,
            followers_count: row.followers_count,
        }
    }
}

impl From<CurrentUserRow> for CurrentUser {
    fn from(row: CurrentUserRow) -> Self {
        Self {
            id: row.id,
            username: row.username,
            nickname: row.nickname,
            avatar: row.avatar,
            phone_number: row.phone_number,
            activation_code: row.activation_code,
            balance: row.balance,
            is_admin: row.is_admin,
            status: row.status,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<UserSummaryRow> for UserSummary {
    fn from(row: UserSummaryRow) -> Self {
        Self {
            id: row.id,
            username: row.username,
            status: row.status,
        }
    }
}

impl From<UserPreviewRow> for UserPreview {
    fn from(row: UserPreviewRow) -> Self {
        Self {
            id: row.id,
            username: row.username,
            nickname: row.nickname,
            avatar: row.avatar,
            created_at: row.created_at,
        }
    }
}
