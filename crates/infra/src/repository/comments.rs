use chrono::{DateTime, Utc};
use evt_domain::{AppError, CommentContentItem, CommentSummary, PagedResponse};
use sqlx::{FromRow, MySql, MySqlPool, QueryBuilder};

use super::map_db_error;

#[derive(Clone)]
pub struct CommentRepository {
    pool: MySqlPool,
}

impl CommentRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        post_id: i64,
        user_id: i64,
        content: &str,
    ) -> Result<CommentSummary, AppError> {
        let result = sqlx::query(
            r#"
            INSERT INTO comments (post_id, user_id, content)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(post_id)
        .bind(user_id)
        .bind(content)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        self.find_by_id(result.last_insert_id() as i64)
            .await?
            .ok_or_else(|| AppError::Internal("created comment cannot be loaded".into()))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<CommentSummary>, AppError> {
        sqlx::query_as::<_, CommentRow>(
            r#"
            SELECT c.id, c.post_id, c.user_id, u.username, c.content, c.created_at
            FROM comments c
            INNER JOIN users u ON u.id = c.user_id
            WHERE c.id = ?
            LIMIT 1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Into::into))
        .map_err(map_db_error)
    }

    pub async fn delete_by_id(&self, id: i64) -> Result<(), AppError> {
        sqlx::query(
            r#"
            DELETE FROM comments
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn list_by_post(
        &self,
        post_id: i64,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<CommentSummary>, AppError> {
        self.list_by_post_with_style(post_id, "default", page, page_size)
            .await
    }

    pub async fn list_by_post_with_style(
        &self,
        post_id: i64,
        style: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<CommentSummary>, AppError> {
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let page_size_i64 = page_size as i64;
        let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM comments WHERE post_id = ?")
            .bind(post_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;

        let order_by = comment_sort_clause(style);
        let sql = format!(
            r#"
            SELECT c.id, c.post_id, c.user_id, u.username, c.content, c.created_at
            FROM comments c
            INNER JOIN users u ON u.id = c.user_id
            LEFT JOIN (
                SELECT comment_id, COUNT(*) AS reply_count
                FROM comment_replies
                GROUP BY comment_id
            ) reply_stats ON reply_stats.comment_id = c.id
            LEFT JOIN (
                SELECT
                  comment_id,
                  COALESCE(SUM(CASE WHEN is_thumbs_up THEN 1 ELSE 0 END), 0) AS thumbs_up_count,
                  COALESCE(SUM(CASE WHEN is_thumbs_down THEN 1 ELSE 0 END), 0) AS thumbs_down_count
                FROM comment_reactions
                WHERE target_type = 0
                GROUP BY comment_id
            ) reaction_stats ON reaction_stats.comment_id = c.id
            LEFT JOIN legacy_comment_states lcs ON lcs.comment_id = c.id
            WHERE c.post_id = ?
            ORDER BY {order_by}
            LIMIT ? OFFSET ?
            "#
        );

        let items = sqlx::query_as::<_, CommentRow>(&sql)
            .bind(post_id)
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

    pub async fn create_content(
        &self,
        comment_id: i64,
        user_id: i64,
        content: &str,
        content_type: i32,
        sort: i64,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO comment_contents (comment_id, user_id, content_type, content, sort_order)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(comment_id)
        .bind(user_id)
        .bind(content_type)
        .bind(content)
        .bind(sort)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn list_contents_by_comment_ids(
        &self,
        comment_ids: &[i64],
    ) -> Result<Vec<CommentContentItem>, AppError> {
        if comment_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut builder: QueryBuilder<MySql> = QueryBuilder::new(
            "SELECT id, comment_id, user_id, content_type, content, sort_order, created_at FROM comment_contents WHERE comment_id IN (",
        );
        let mut separated = builder.separated(", ");
        for comment_id in comment_ids {
            separated.push_bind(comment_id);
        }
        separated.push_unseparated(") ORDER BY comment_id ASC, sort_order ASC, id ASC");

        builder
            .build_query_as::<CommentContentRow>()
            .fetch_all(&self.pool)
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(map_db_error)
    }
}

fn comment_sort_clause(style: &str) -> &'static str {
    match style {
        "hots" => {
            "COALESCE(lcs.is_essence, FALSE) DESC, (COALESCE(reply_stats.reply_count, 0) * 2 + COALESCE(reaction_stats.thumbs_up_count, 0) * 4 - COALESCE(reaction_stats.thumbs_down_count, 0)) DESC, c.id DESC"
        }
        "newest" => "COALESCE(lcs.is_essence, FALSE) DESC, c.id DESC",
        _ => "COALESCE(lcs.is_essence, FALSE) DESC, c.id ASC",
    }
}

#[cfg(test)]
mod tests {
    use super::comment_sort_clause;

    #[test]
    fn legacy_comment_sort_styles_match_expected_clauses() {
        assert!(comment_sort_clause("default").contains("c.id ASC"));
        assert!(comment_sort_clause("newest").contains("c.id DESC"));
        assert!(comment_sort_clause("hots").contains("reply_count, 0) * 2"));
        assert!(comment_sort_clause("hots").contains("thumbs_up_count, 0) * 4"));
        assert!(comment_sort_clause("hots").contains("thumbs_down_count, 0)"));
    }
}

#[derive(Debug, FromRow)]
struct CommentRow {
    id: i64,
    post_id: i64,
    user_id: i64,
    username: String,
    content: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct CommentContentRow {
    id: i64,
    comment_id: i64,
    user_id: i64,
    content_type: i32,
    content: String,
    sort_order: i64,
    created_at: DateTime<Utc>,
}

impl From<CommentRow> for CommentSummary {
    fn from(row: CommentRow) -> Self {
        Self {
            id: row.id,
            post_id: row.post_id,
            user_id: row.user_id,
            username: row.username,
            content: row.content,
            created_at: row.created_at,
        }
    }
}

impl From<CommentContentRow> for CommentContentItem {
    fn from(row: CommentContentRow) -> Self {
        Self {
            id: row.id,
            comment_id: row.comment_id,
            user_id: row.user_id,
            content: row.content,
            content_type: row.content_type,
            sort: row.sort_order,
            created_at: row.created_at,
        }
    }
}
