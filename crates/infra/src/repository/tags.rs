use evt_domain::{AppError, TagSummary};
use sqlx::{FromRow, MySqlPool};

use super::map_db_error;

#[derive(Clone)]
pub struct TagRepository {
    pool: MySqlPool,
}

impl TagRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn record_tags(&self, user_id: i64, tags: &[String]) -> Result<(), AppError> {
        for tag in tags {
            sqlx::query(
                r#"
                INSERT INTO tags (user_id, tag, quote_num)
                VALUES (?, ?, 1)
                ON DUPLICATE KEY UPDATE
                  quote_num = quote_num + 1
                "#,
            )
            .bind(user_id)
            .bind(tag)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        }

        Ok(())
    }

    pub async fn suggest(&self, keyword: &str, limit: u64) -> Result<Vec<String>, AppError> {
        let pattern = format!("%{}%", keyword.trim());
        sqlx::query_scalar::<_, String>(
            r#"
            SELECT tag
            FROM tags
            WHERE tag LIKE ?
            ORDER BY quote_num DESC, id DESC
            LIMIT ?
            "#,
        )
        .bind(pattern)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)
    }

    pub async fn list_hot(&self, limit: u64) -> Result<Vec<TagSummary>, AppError> {
        self.fetch_tags(
            r#"
            SELECT
              t.id,
              t.user_id,
              u.username,
              t.tag,
              t.quote_num,
              t.created_at,
              FALSE AS is_following,
              FALSE AS is_top,
              FALSE AS is_pin
            FROM tags t
            INNER JOIN users u ON u.id = t.user_id
            ORDER BY t.quote_num DESC, t.id DESC
            LIMIT ?
            "#,
            limit,
        )
        .await
    }

    pub async fn list_new(&self, limit: u64) -> Result<Vec<TagSummary>, AppError> {
        self.fetch_tags(
            r#"
            SELECT
              t.id,
              t.user_id,
              u.username,
              t.tag,
              t.quote_num,
              t.created_at,
              FALSE AS is_following,
              FALSE AS is_top,
              FALSE AS is_pin
            FROM tags t
            INNER JOIN users u ON u.id = t.user_id
            ORDER BY t.created_at DESC, t.id DESC
            LIMIT ?
            "#,
            limit,
        )
        .await
    }

    pub async fn list_followed(
        &self,
        user_id: i64,
        limit: u64,
    ) -> Result<Vec<TagSummary>, AppError> {
        self.fetch_followed(
            r#"
            SELECT
              t.id,
              t.user_id,
              u.username,
              t.tag,
              t.quote_num,
              t.created_at,
              TRUE AS is_following,
              tu.is_top,
              tu.is_pin
            FROM topic_users tu
            INNER JOIN tags t ON t.id = tu.tag_id
            INNER JOIN users u ON u.id = t.user_id
            WHERE tu.user_id = ?
            ORDER BY tu.is_top DESC, tu.updated_at DESC, tu.id DESC
            LIMIT ?
            "#,
            user_id,
            limit,
        )
        .await
    }

    pub async fn list_pinned(&self, user_id: i64, limit: u64) -> Result<Vec<TagSummary>, AppError> {
        self.fetch_followed(
            r#"
            SELECT
              t.id,
              t.user_id,
              u.username,
              t.tag,
              t.quote_num,
              t.created_at,
              TRUE AS is_following,
              tu.is_top,
              tu.is_pin
            FROM topic_users tu
            INNER JOIN tags t ON t.id = tu.tag_id
            INNER JOIN users u ON u.id = t.user_id
            WHERE tu.user_id = ? AND tu.is_pin = TRUE
            ORDER BY tu.updated_at DESC, tu.id DESC
            LIMIT ?
            "#,
            user_id,
            limit,
        )
        .await
    }

    pub async fn list_hot_with_followed(
        &self,
        user_id: Option<i64>,
        hot_limit: u64,
        extra_limit: u64,
    ) -> Result<(Vec<TagSummary>, Vec<TagSummary>), AppError> {
        let hot = self.list_hot(hot_limit).await?;
        let extra = match user_id {
            Some(user_id) => self.list_followed(user_id, extra_limit).await?,
            None => Vec::new(),
        };
        Ok((hot, extra))
    }

    pub async fn follow(&self, user_id: i64, tag_id: i64) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO topic_users (user_id, tag_id, is_top, is_pin)
            VALUES (?, ?, FALSE, FALSE)
            ON DUPLICATE KEY UPDATE
              updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(user_id)
        .bind(tag_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn unfollow(&self, user_id: i64, tag_id: i64) -> Result<(), AppError> {
        sqlx::query(
            r#"
            DELETE FROM topic_users
            WHERE user_id = ? AND tag_id = ?
            "#,
        )
        .bind(user_id)
        .bind(tag_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn toggle_top(&self, user_id: i64, tag_id: i64) -> Result<bool, AppError> {
        sqlx::query(
            r#"
            UPDATE topic_users
            SET is_top = NOT is_top
            WHERE user_id = ? AND tag_id = ?
            "#,
        )
        .bind(user_id)
        .bind(tag_id)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        self.follow_state(user_id, tag_id, "is_top").await
    }

    pub async fn toggle_pin(&self, user_id: i64, tag_id: i64) -> Result<bool, AppError> {
        sqlx::query(
            r#"
            UPDATE topic_users
            SET is_pin = NOT is_pin
            WHERE user_id = ? AND tag_id = ?
            "#,
        )
        .bind(user_id)
        .bind(tag_id)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        self.follow_state(user_id, tag_id, "is_pin").await
    }

    async fn follow_state(&self, user_id: i64, tag_id: i64, field: &str) -> Result<bool, AppError> {
        let sql =
            format!("SELECT {field} FROM topic_users WHERE user_id = ? AND tag_id = ? LIMIT 1");
        sqlx::query_scalar::<_, bool>(&sql)
            .bind(user_id)
            .bind(tag_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn fetch_tags(&self, sql: &str, limit: u64) -> Result<Vec<TagSummary>, AppError> {
        sqlx::query_as::<_, TagRow>(sql)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
            .map(|rows| rows.into_iter().map(Into::into).collect())
            .map_err(map_db_error)
    }

    async fn fetch_followed(
        &self,
        sql: &str,
        user_id: i64,
        limit: u64,
    ) -> Result<Vec<TagSummary>, AppError> {
        sqlx::query_as::<_, TagRow>(sql)
            .bind(user_id)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
            .map(|rows| rows.into_iter().map(Into::into).collect())
            .map_err(map_db_error)
    }
}

#[derive(Debug, FromRow)]
struct TagRow {
    id: i64,
    user_id: i64,
    username: String,
    tag: String,
    quote_num: i64,
    created_at: chrono::DateTime<chrono::Utc>,
    is_following: bool,
    is_top: bool,
    is_pin: bool,
}

impl From<TagRow> for TagSummary {
    fn from(row: TagRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            username: row.username,
            tag: row.tag,
            quote_num: row.quote_num,
            created_at: row.created_at,
            is_following: row.is_following,
            is_top: row.is_top,
            is_pin: row.is_pin,
        }
    }
}
