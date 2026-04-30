use std::collections::{HashMap, HashSet};

use evt_domain::{AppError, PagedResponse, UserPreview, UserSummary};
use sqlx::{FromRow, MySql, MySqlPool, QueryBuilder};

use super::map_db_error;

#[derive(Clone)]
pub struct FollowRepository {
    pool: MySqlPool,
}

impl FollowRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn follow(&self, follower_id: i64, followee_id: i64) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO follows (follower_id, followee_id)
            VALUES (?, ?)
            "#,
        )
        .bind(follower_id)
        .bind(followee_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn unfollow(&self, follower_id: i64, followee_id: i64) -> Result<(), AppError> {
        sqlx::query(
            r#"
            DELETE FROM follows
            WHERE follower_id = ? AND followee_id = ?
            "#,
        )
        .bind(follower_id)
        .bind(followee_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn is_following(&self, follower_id: i64, followee_id: i64) -> Result<bool, AppError> {
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM follows
            WHERE follower_id = ? AND followee_id = ?
            "#,
        )
        .bind(follower_id)
        .bind(followee_id)
        .fetch_one(&self.pool)
        .await
        .map(|count| count > 0)
        .map_err(map_db_error)
    }

    pub async fn batch_following_status(
        &self,
        follower_id: i64,
        followee_ids: &[i64],
    ) -> Result<HashMap<i64, bool>, AppError> {
        if followee_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut builder: QueryBuilder<MySql> =
            QueryBuilder::new("SELECT followee_id FROM follows WHERE follower_id = ");
        builder.push_bind(follower_id);
        builder.push(" AND followee_id IN (");
        let mut separated = builder.separated(", ");
        for followee_id in followee_ids {
            separated.push_bind(followee_id);
        }
        separated.push_unseparated(")");

        let followed_ids = builder
            .build_query_scalar::<i64>()
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_error)?
            .into_iter()
            .collect::<HashSet<_>>();

        Ok(followee_ids
            .iter()
            .copied()
            .map(|id| (id, followed_ids.contains(&id)))
            .collect())
    }

    pub async fn list_followers(
        &self,
        username: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<UserSummary>, AppError> {
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let page_size_i64 = page_size as i64;

        let total = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM follows f
            INNER JOIN users target ON target.id = f.followee_id
            WHERE target.username = ?
            "#,
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        let items = sqlx::query_as::<_, UserSummaryRow>(
            r#"
            SELECT u.id, u.username, u.status
            FROM follows f
            INNER JOIN users target ON target.id = f.followee_id
            INNER JOIN users u ON u.id = f.follower_id
            WHERE target.username = ?
            ORDER BY f.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(username)
        .bind(page_size_i64)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?
        .into_iter()
        .map(Into::into)
        .collect();

        Ok(PagedResponse {
            items,
            total,
            page,
            page_size,
        })
    }

    pub async fn list_followings(
        &self,
        username: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<UserSummary>, AppError> {
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let page_size_i64 = page_size as i64;

        let total = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM follows f
            INNER JOIN users target ON target.id = f.follower_id
            WHERE target.username = ?
            "#,
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        let items = sqlx::query_as::<_, UserSummaryRow>(
            r#"
            SELECT u.id, u.username, u.status
            FROM follows f
            INNER JOIN users target ON target.id = f.follower_id
            INNER JOIN users u ON u.id = f.followee_id
            WHERE target.username = ?
            ORDER BY f.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(username)
        .bind(page_size_i64)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?
        .into_iter()
        .map(Into::into)
        .collect();

        Ok(PagedResponse {
            items,
            total,
            page,
            page_size,
        })
    }

    pub async fn list_follower_previews(
        &self,
        username: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<UserPreview>, AppError> {
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let page_size_i64 = page_size as i64;

        let total = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM follows f
            INNER JOIN users target ON target.id = f.followee_id
            WHERE target.username = ?
            "#,
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        let items = sqlx::query_as::<_, UserPreviewRow>(
            r#"
            SELECT u.id, u.username, COALESCE(up.nickname, u.username) AS nickname, COALESCE(up.avatar, '') AS avatar, u.created_at
            FROM follows f
            INNER JOIN users target ON target.id = f.followee_id
            INNER JOIN users u ON u.id = f.follower_id
            LEFT JOIN user_profiles up ON up.user_id = u.id
            WHERE target.username = ?
            ORDER BY f.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(username)
        .bind(page_size_i64)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?
        .into_iter()
        .map(Into::into)
        .collect();

        Ok(PagedResponse {
            items,
            total,
            page,
            page_size,
        })
    }

    pub async fn list_following_previews(
        &self,
        username: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<UserPreview>, AppError> {
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let page_size_i64 = page_size as i64;

        let total = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM follows f
            INNER JOIN users target ON target.id = f.follower_id
            WHERE target.username = ?
            "#,
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        let items = sqlx::query_as::<_, UserPreviewRow>(
            r#"
            SELECT u.id, u.username, COALESCE(up.nickname, u.username) AS nickname, COALESCE(up.avatar, '') AS avatar, u.created_at
            FROM follows f
            INNER JOIN users target ON target.id = f.follower_id
            INNER JOIN users u ON u.id = f.followee_id
            LEFT JOIN user_profiles up ON up.user_id = u.id
            WHERE target.username = ?
            ORDER BY f.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(username)
        .bind(page_size_i64)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?
        .into_iter()
        .map(Into::into)
        .collect();

        Ok(PagedResponse {
            items,
            total,
            page,
            page_size,
        })
    }
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
    created_at: chrono::DateTime<chrono::Utc>,
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
