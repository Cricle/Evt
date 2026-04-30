use std::collections::HashMap;

use paopao_domain::AppError;
use sqlx::{MySql, MySqlPool, QueryBuilder, Row};

use super::map_db_error;

pub const FRIEND_STATUS_REQUESTING: i32 = 1;
pub const FRIEND_STATUS_AGREE: i32 = 2;
pub const FRIEND_STATUS_REJECT: i32 = 3;
pub const FRIEND_STATUS_DELETED: i32 = 4;

#[derive(Clone)]
pub struct FriendshipRepository {
    pool: MySqlPool,
}

impl FriendshipRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn request(&self, user_id: i64, friend_id: i64) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO friendships (user_id, friend_id, status)
            VALUES (?, ?, ?)
            ON DUPLICATE KEY UPDATE
              status = VALUES(status),
              updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(user_id)
        .bind(friend_id)
        .bind(FRIEND_STATUS_REQUESTING)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn approve(&self, user_id: i64, friend_id: i64) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(map_db_error)?;

        sqlx::query(
            r#"
            UPDATE friendships
            SET status = ?, updated_at = CURRENT_TIMESTAMP
            WHERE user_id = ? AND friend_id = ?
            "#,
        )
        .bind(FRIEND_STATUS_AGREE)
        .bind(friend_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;

        sqlx::query(
            r#"
            INSERT INTO friendships (user_id, friend_id, status)
            VALUES (?, ?, ?)
            ON DUPLICATE KEY UPDATE
              status = VALUES(status),
              updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(user_id)
        .bind(friend_id)
        .bind(FRIEND_STATUS_AGREE)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;

        tx.commit().await.map_err(map_db_error)
    }

    pub async fn reject(&self, user_id: i64, friend_id: i64) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE friendships
            SET status = ?, updated_at = CURRENT_TIMESTAMP
            WHERE user_id = ? AND friend_id = ?
            "#,
        )
        .bind(FRIEND_STATUS_REJECT)
        .bind(friend_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn delete_pair(&self, user_id: i64, friend_id: i64) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE friendships
            SET status = ?, updated_at = CURRENT_TIMESTAMP
            WHERE (user_id = ? AND friend_id = ?)
               OR (user_id = ? AND friend_id = ?)
            "#,
        )
        .bind(FRIEND_STATUS_DELETED)
        .bind(user_id)
        .bind(friend_id)
        .bind(friend_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn is_friend(&self, user_id: i64, friend_id: i64) -> Result<bool, AppError> {
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM friendships
            WHERE user_id = ? AND friend_id = ? AND status = ?
            "#,
        )
        .bind(user_id)
        .bind(friend_id)
        .bind(FRIEND_STATUS_AGREE)
        .fetch_one(&self.pool)
        .await
        .map(|count| count > 0)
        .map_err(map_db_error)
    }

    pub async fn batch_friend_status(
        &self,
        user_id: i64,
        friend_ids: &[i64],
    ) -> Result<HashMap<i64, bool>, AppError> {
        if friend_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut builder: QueryBuilder<MySql> =
            QueryBuilder::new("SELECT friend_id FROM friendships WHERE user_id = ");
        builder.push_bind(user_id);
        builder.push(" AND status = ");
        builder.push_bind(FRIEND_STATUS_AGREE);
        builder.push(" AND friend_id IN (");
        let mut separated = builder.separated(", ");
        for friend_id in friend_ids {
            separated.push_bind(friend_id);
        }
        separated.push_unseparated(")");

        builder
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_error)
            .map(|rows| {
                rows.into_iter()
                    .map(|row| row.get::<i64, _>("friend_id"))
                    .map(|friend_id| (friend_id, true))
                    .collect()
            })
    }
}
