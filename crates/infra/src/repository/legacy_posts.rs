use std::collections::HashMap;

use evt_domain::{AppError, LegacyCommentState, LegacyPostState};
use sqlx::{FromRow, MySql, MySqlPool, QueryBuilder, Row};

use super::map_db_error;

pub const REACTION_TARGET_COMMENT: i32 = 0;

#[derive(Clone)]
pub struct LegacyPostRepository {
    pool: MySqlPool,
}

impl LegacyPostRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn ensure_post_state(
        &self,
        post_id: i64,
        attachment_price: i64,
        visibility: i32,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO legacy_post_states (post_id, attachment_price, visibility)
            VALUES (?, ?, ?)
            ON DUPLICATE KEY UPDATE
              attachment_price = VALUES(attachment_price),
              visibility = VALUES(visibility),
              updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(post_id)
        .bind(attachment_price)
        .bind(visibility)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn post_states_by_ids(
        &self,
        post_ids: &[i64],
    ) -> Result<HashMap<i64, LegacyPostState>, AppError> {
        if post_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut builder: QueryBuilder<MySql> = QueryBuilder::new(
            "SELECT post_id, attachment_price, visibility, is_lock, is_top, is_essence FROM legacy_post_states WHERE post_id IN (",
        );
        let mut separated = builder.separated(", ");
        for post_id in post_ids {
            separated.push_bind(post_id);
        }
        separated.push_unseparated(")");

        builder
            .build_query_as::<LegacyPostStateRow>()
            .fetch_all(&self.pool)
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(Into::into)
                    .map(|item: LegacyPostState| (item.post_id, item))
                    .collect()
            })
            .map_err(map_db_error)
    }

    pub async fn toggle_post_flag(&self, post_id: i64, field: &str) -> Result<bool, AppError> {
        let sql = format!(
            "UPDATE legacy_post_states SET {field} = NOT {field}, updated_at = CURRENT_TIMESTAMP WHERE post_id = ?"
        );
        sqlx::query(&sql)
            .bind(post_id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;

        let sql = format!("SELECT {field} FROM legacy_post_states WHERE post_id = ? LIMIT 1");
        sqlx::query_scalar::<_, bool>(&sql)
            .bind(post_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)
    }

    pub async fn set_post_visibility(
        &self,
        post_id: i64,
        visibility: i32,
    ) -> Result<i32, AppError> {
        sqlx::query(
            r#"
            UPDATE legacy_post_states
            SET visibility = ?, updated_at = CURRENT_TIMESTAMP
            WHERE post_id = ?
            "#,
        )
        .bind(visibility)
        .bind(post_id)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        sqlx::query_scalar::<_, i32>(
            "SELECT visibility FROM legacy_post_states WHERE post_id = ? LIMIT 1",
        )
        .bind(post_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    pub async fn ensure_comment_state(&self, comment_id: i64) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO legacy_comment_states (comment_id, is_essence, is_reaction)
            VALUES (?, FALSE, FALSE)
            ON DUPLICATE KEY UPDATE comment_id = comment_id
            "#,
        )
        .bind(comment_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn comment_states_by_ids(
        &self,
        comment_ids: &[i64],
    ) -> Result<HashMap<i64, LegacyCommentState>, AppError> {
        if comment_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut builder: QueryBuilder<MySql> = QueryBuilder::new(
            "SELECT comment_id, is_essence, is_reaction FROM legacy_comment_states WHERE comment_id IN (",
        );
        let mut separated = builder.separated(", ");
        for comment_id in comment_ids {
            separated.push_bind(comment_id);
        }
        separated.push_unseparated(")");

        builder
            .build_query_as::<LegacyCommentStateRow>()
            .fetch_all(&self.pool)
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(Into::into)
                    .map(|item: LegacyCommentState| (item.comment_id, item))
                    .collect()
            })
            .map_err(map_db_error)
    }

    pub async fn toggle_comment_essence(&self, comment_id: i64) -> Result<bool, AppError> {
        sqlx::query(
            r#"
            UPDATE legacy_comment_states
            SET is_essence = NOT is_essence, updated_at = CURRENT_TIMESTAMP
            WHERE comment_id = ?
            "#,
        )
        .bind(comment_id)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        sqlx::query_scalar::<_, bool>(
            "SELECT is_essence FROM legacy_comment_states WHERE comment_id = ? LIMIT 1",
        )
        .bind(comment_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    pub async fn set_comment_reaction(
        &self,
        comment_id: i64,
        is_reaction: bool,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE legacy_comment_states
            SET is_reaction = ?, updated_at = CURRENT_TIMESTAMP
            WHERE comment_id = ?
            "#,
        )
        .bind(is_reaction)
        .bind(comment_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn toggle_reaction(
        &self,
        user_id: i64,
        post_id: i64,
        comment_id: i64,
        thumbs_up: bool,
    ) -> Result<(), AppError> {
        let existing = sqlx::query(
            r#"
            SELECT is_thumbs_up, is_thumbs_down
            FROM comment_reactions
            WHERE user_id = ? AND comment_id = ?
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .bind(comment_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        let (next_up, next_down) = match existing {
            Some(row) => {
                let is_up = row.get::<bool, _>("is_thumbs_up");
                let is_down = row.get::<bool, _>("is_thumbs_down");
                if thumbs_up {
                    (!is_up, false)
                } else {
                    (false, !is_down)
                }
            }
            None => (thumbs_up, !thumbs_up),
        };

        sqlx::query(
            r#"
            INSERT INTO comment_reactions (user_id, post_id, comment_id, is_thumbs_up, is_thumbs_down)
            VALUES (?, ?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE
              is_thumbs_up = VALUES(is_thumbs_up),
              is_thumbs_down = VALUES(is_thumbs_down),
              updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(user_id)
        .bind(post_id)
        .bind(comment_id)
        .bind(next_up)
        .bind(next_down)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn reaction_counts_by_comments(
        &self,
        comment_ids: &[i64],
    ) -> Result<HashMap<i64, i64>, AppError> {
        self.reaction_counts(comment_ids, "comment_id")
            .await
    }

    async fn reaction_counts(
        &self,
        ids: &[i64],
        id_column: &str,
    ) -> Result<HashMap<i64, i64>, AppError> {
        if ids.is_empty() {
            return Ok(HashMap::new());
        }

        let sql = format!(
            "SELECT {id_column} AS target_id, COALESCE(SUM(CASE WHEN is_thumbs_up THEN 1 ELSE 0 END), 0) AS thumbs_up_count FROM comment_reactions WHERE {id_column} IN ("
        );
        let mut builder: QueryBuilder<MySql> = QueryBuilder::new(&sql);
        let mut separated = builder.separated(", ");
        for id in ids {
            separated.push_bind(id);
        }
        separated.push_unseparated(") GROUP BY target_id");

        builder
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_error)
            .map(|rows| {
                rows.into_iter()
                    .map(|row| {
                        (
                            row.get::<i64, _>("target_id"),
                            row.get::<i64, _>("thumbs_up_count"),
                        )
                    })
                    .collect()
            })
    }

    pub async fn reaction_status_map(
        &self,
        user_id: i64,
        comment_ids: &[i64],
    ) -> Result<HashMap<(i32, i64), (bool, bool)>, AppError> {
        let mut result = HashMap::new();

        if !comment_ids.is_empty() {
            let mut builder: QueryBuilder<MySql> = QueryBuilder::new(
                "SELECT comment_id, is_thumbs_up, is_thumbs_down FROM comment_reactions WHERE user_id = ",
            );
            builder.push_bind(user_id);
            builder.push(" AND comment_id IN (");
            let mut separated = builder.separated(", ");
            for id in comment_ids {
                separated.push_bind(id);
            }
            separated.push_unseparated(")");

            for row in builder
                .build()
                .fetch_all(&self.pool)
                .await
                .map_err(map_db_error)?
            {
                result.insert(
                    (REACTION_TARGET_COMMENT, row.get::<i64, _>("comment_id")),
                    (
                        row.get::<bool, _>("is_thumbs_up"),
                        row.get::<bool, _>("is_thumbs_down"),
                    ),
                );
            }
        }

        Ok(result)
    }
}

#[derive(Debug, FromRow)]
struct LegacyPostStateRow {
    post_id: i64,
    attachment_price: i64,
    visibility: i32,
    is_lock: bool,
    is_top: bool,
    is_essence: bool,
}

#[derive(Debug, FromRow)]
struct LegacyCommentStateRow {
    comment_id: i64,
    is_essence: bool,
    is_reaction: bool,
}

impl From<LegacyPostStateRow> for LegacyPostState {
    fn from(row: LegacyPostStateRow) -> Self {
        Self {
            post_id: row.post_id,
            attachment_price: row.attachment_price,
            visibility: row.visibility,
            is_lock: row.is_lock,
            is_top: row.is_top,
            is_essence: row.is_essence,
        }
    }
}

impl From<LegacyCommentStateRow> for LegacyCommentState {
    fn from(row: LegacyCommentStateRow) -> Self {
        Self {
            comment_id: row.comment_id,
            is_essence: row.is_essence,
            is_reaction: row.is_reaction,
        }
    }
}

#[cfg(test)]
mod tests {
    use sqlx::{Execute, MySql, QueryBuilder};

    #[test]
    fn reaction_counts_query_keeps_bind_placeholders_valid() {
        let id_column = "comment_id";
        let sql = format!(
            "SELECT {id_column} AS target_id, COALESCE(SUM(CASE WHEN is_thumbs_up THEN 1 ELSE 0 END), 0) AS thumbs_up_count FROM comment_reactions WHERE {id_column} IN ("
        );
        let mut builder: QueryBuilder<MySql> = QueryBuilder::new(&sql);
        let mut separated = builder.separated(", ");
        separated.push_bind(1_i64);
        separated.push_bind(2_i64);
        separated.push_unseparated(") GROUP BY target_id");

        let built = builder.build();
        let query_text = built.sql();

        assert!(query_text.contains("WHERE comment_id IN (?, ?)"));
        assert!(query_text.ends_with("GROUP BY target_id"));
    }
}
