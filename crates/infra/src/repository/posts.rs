use chrono::{DateTime, Utc};
use evt_domain::{AppError, PagedResponse, PostContentItem, PostSummary};
use sqlx::{FromRow, MySql, MySqlPool, QueryBuilder};

use super::map_db_error;

const POST_SUMMARY_SELECT: &str = r#"
    SELECT
      p.id,
      p.user_id,
      u.username,
      p.content,
      p.tags,
      COALESCE(comment_stats.comments_count, 0) AS comments_count,
      COALESCE(star_stats.upvote_count, 0) AS upvote_count,
      COALESCE(collection_stats.collection_count, 0) AS collection_count,
      p.created_at
"#;

const POST_SUMMARY_FROM: &str = r#"
    FROM posts p
    INNER JOIN users u ON u.id = p.user_id
    LEFT JOIN (
        SELECT post_id, COUNT(*) AS comments_count
        FROM comments
        GROUP BY post_id
    ) comment_stats ON comment_stats.post_id = p.id
    LEFT JOIN (
        SELECT post_id, COUNT(*) AS upvote_count
        FROM post_stars
        GROUP BY post_id
    ) star_stats ON star_stats.post_id = p.id
    LEFT JOIN (
        SELECT post_id, COUNT(*) AS collection_count
        FROM post_collections
        GROUP BY post_id
    ) collection_stats ON collection_stats.post_id = p.id
"#;

fn post_summary_query(suffix: &str) -> String {
    format!("{POST_SUMMARY_SELECT}{POST_SUMMARY_FROM}{suffix}")
}

#[derive(Clone)]
pub struct PostRepository {
    pool: MySqlPool,
}

impl PostRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        user_id: i64,
        content: &str,
        tags: &str,
    ) -> Result<PostSummary, AppError> {
        let result = sqlx::query(
            r#"
            INSERT INTO posts (user_id, content, tags)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(user_id)
        .bind(content)
        .bind(tags)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        self.find_by_id(result.last_insert_id() as i64)
            .await?
            .ok_or_else(|| AppError::Internal("created post cannot be loaded".into()))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<PostSummary>, AppError> {
        sqlx::query_as::<_, PostRow>(&post_summary_query(" WHERE p.id = ? LIMIT 1"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map(|row| row.map(Into::into))
            .map_err(map_db_error)
    }

    pub async fn update_content(
        &self,
        id: i64,
        content: &str,
    ) -> Result<Option<PostSummary>, AppError> {
        sqlx::query(
            r#"
            UPDATE posts
            SET content = ?, tags = ?
            WHERE id = ?
            "#,
        )
        .bind(content)
        .bind("")
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        self.find_by_id(id).await
    }

    pub async fn delete_by_id(&self, id: i64) -> Result<(), AppError> {
        sqlx::query(
            r#"
            DELETE FROM posts
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn list(
        &self,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let page_size_i64 = page_size as i64;

        let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM posts")
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;

        let items = sqlx::query_as::<_, PostRow>(&post_summary_query(
            " ORDER BY p.id DESC LIMIT ? OFFSET ?",
        ))
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

    pub async fn list_hot(
        &self,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let page_size_i64 = page_size as i64;
        let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM posts")
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;

        let items = sqlx::query_as::<_, PostRow>(&post_summary_query(
            " ORDER BY upvote_count DESC, comments_count DESC, collection_count DESC, p.id DESC LIMIT ? OFFSET ?",
        ))
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

    pub async fn search(
        &self,
        query: &str,
        query_type: Option<&str>,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let page_size_i64 = page_size as i64;
        let query = query.trim();

        if matches!(query_type, Some("tag")) {
            let total = sqlx::query_scalar::<_, i64>(
                r#"
                SELECT COUNT(*)
                FROM posts
                WHERE FIND_IN_SET(?, tags) > 0
                "#,
            )
            .bind(query)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;

            let items = sqlx::query_as::<_, PostRow>(&post_summary_query(
                " WHERE FIND_IN_SET(?, p.tags) > 0 ORDER BY p.id DESC LIMIT ? OFFSET ?",
            ))
            .bind(query)
            .bind(page_size_i64)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_error)?
            .into_iter()
            .map(Into::into)
            .collect();

            return Ok(PagedResponse {
                items,
                total,
                page,
                page_size,
            });
        }

        let pattern = format!("%{query}%");
        let total = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM posts p
            INNER JOIN users u ON u.id = p.user_id
            WHERE p.content LIKE ? OR u.username LIKE ? OR p.tags LIKE ?
            "#,
        )
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        let items = sqlx::query_as::<_, PostRow>(&post_summary_query(
            " WHERE p.content LIKE ? OR u.username LIKE ? OR p.tags LIKE ? ORDER BY p.id DESC LIMIT ? OFFSET ?",
        ))
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
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

    pub async fn list_by_username(
        &self,
        username: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let page_size_i64 = page_size as i64;

        let total = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM posts p
            INNER JOIN users u ON u.id = p.user_id
            WHERE u.username = ?
            "#,
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        let items = sqlx::query_as::<_, PostRow>(&post_summary_query(
            " WHERE u.username = ? ORDER BY p.id DESC LIMIT ? OFFSET ?",
        ))
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

    pub async fn list_highlighted_by_username(
        &self,
        username: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        self.list_by_username_with_exists(
            username,
            page,
            page_size,
            "EXISTS (SELECT 1 FROM legacy_post_states lps WHERE lps.post_id = p.id AND lps.is_essence = TRUE)",
        )
        .await
    }

    pub async fn list_media_by_username(
        &self,
        username: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        self.list_by_username_with_exists(
            username,
            page,
            page_size,
            "EXISTS (SELECT 1 FROM post_contents pc WHERE pc.post_id = p.id AND pc.content_type IN (3, 4, 5, 7, 8))",
        )
        .await
    }

    pub async fn list_commented_posts_by_username(
        &self,
        username: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let page_size_i64 = page_size as i64;

        let total = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM posts p
            WHERE EXISTS (
                SELECT 1
                FROM comments c
                INNER JOIN users cu ON cu.id = c.user_id
                WHERE c.post_id = p.id AND cu.username = ?
            )
            "#,
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        let items = sqlx::query_as::<_, PostRow>(&post_summary_query(
            " WHERE EXISTS (SELECT 1 FROM comments c INNER JOIN users cu ON cu.id = c.user_id WHERE c.post_id = p.id AND cu.username = ?) ORDER BY p.id DESC LIMIT ? OFFSET ?",
        ))
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

    pub async fn list_feed(
        &self,
        actor_id: i64,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let page_size_i64 = page_size as i64;

        let total = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM posts p
            WHERE p.user_id = ?
               OR p.user_id IN (
                    SELECT followee_id
                    FROM follows
                    WHERE follower_id = ?
               )
            "#,
        )
        .bind(actor_id)
        .bind(actor_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        let items = sqlx::query_as::<_, PostRow>(&post_summary_query(
            " WHERE p.user_id = ? OR p.user_id IN (SELECT followee_id FROM follows WHERE follower_id = ?) ORDER BY p.id DESC LIMIT ? OFFSET ?",
        ))
        .bind(actor_id)
        .bind(actor_id)
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
        post_id: i64,
        user_id: i64,
        content: &str,
        content_type: i32,
        sort: i64,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO post_contents (post_id, user_id, content_type, content, sort_order)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(post_id)
        .bind(user_id)
        .bind(content_type)
        .bind(content)
        .bind(sort)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn has_star(&self, post_id: i64, user_id: i64) -> Result<bool, AppError> {
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM post_stars
            WHERE post_id = ? AND user_id = ?
            "#,
        )
        .bind(post_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map(|count| count > 0)
        .map_err(map_db_error)
    }

    pub async fn create_star(&self, post_id: i64, user_id: i64) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO post_stars (post_id, user_id)
            VALUES (?, ?)
            "#,
        )
        .bind(post_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn delete_star(&self, post_id: i64, user_id: i64) -> Result<(), AppError> {
        sqlx::query(
            r#"
            DELETE FROM post_stars
            WHERE post_id = ? AND user_id = ?
            "#,
        )
        .bind(post_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn has_collection(&self, post_id: i64, user_id: i64) -> Result<bool, AppError> {
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM post_collections
            WHERE post_id = ? AND user_id = ?
            "#,
        )
        .bind(post_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map(|count| count > 0)
        .map_err(map_db_error)
    }

    pub async fn create_collection(&self, post_id: i64, user_id: i64) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO post_collections (post_id, user_id)
            VALUES (?, ?)
            "#,
        )
        .bind(post_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn delete_collection(&self, post_id: i64, user_id: i64) -> Result<(), AppError> {
        sqlx::query(
            r#"
            DELETE FROM post_collections
            WHERE post_id = ? AND user_id = ?
            "#,
        )
        .bind(post_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn list_collections_by_user_id(
        &self,
        user_id: i64,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let page_size_i64 = page_size as i64;

        let total = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM post_collections
            WHERE user_id = ?
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        let items = sqlx::query_as::<_, PostRow>(&post_summary_query(
            " INNER JOIN post_collections current_collection ON current_collection.post_id = p.id WHERE current_collection.user_id = ? ORDER BY current_collection.id DESC LIMIT ? OFFSET ?",
        ))
        .bind(user_id)
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

    pub async fn list_stars_by_username(
        &self,
        username: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let page_size_i64 = page_size as i64;

        let total = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM post_stars ps
            INNER JOIN users starred_by ON starred_by.id = ps.user_id
            WHERE starred_by.username = ?
            "#,
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        let items = sqlx::query_as::<_, PostRow>(&post_summary_query(
            " INNER JOIN post_stars current_star ON current_star.post_id = p.id INNER JOIN users starred_by ON starred_by.id = current_star.user_id WHERE starred_by.username = ? ORDER BY current_star.id DESC LIMIT ? OFFSET ?",
        ))
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

    async fn list_by_username_with_exists(
        &self,
        username: &str,
        page: u64,
        page_size: u64,
        exists_sql: &str,
    ) -> Result<PagedResponse<PostSummary>, AppError> {
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let page_size_i64 = page_size as i64;

        let total_sql = format!(
            "SELECT COUNT(*) FROM posts p INNER JOIN users u ON u.id = p.user_id WHERE u.username = ? AND {exists_sql}"
        );
        let total = sqlx::query_scalar::<_, i64>(&total_sql)
            .bind(username)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;

        let list_sql = post_summary_query(&format!(
            " WHERE u.username = ? AND {exists_sql} ORDER BY p.id DESC LIMIT ? OFFSET ?"
        ));
        let items = sqlx::query_as::<_, PostRow>(&list_sql)
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

    pub async fn list_contents_by_post_ids(
        &self,
        post_ids: &[i64],
    ) -> Result<Vec<PostContentItem>, AppError> {
        if post_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut builder: QueryBuilder<MySql> = QueryBuilder::new(
            "SELECT id, post_id, user_id, content_type, content, sort_order, created_at FROM post_contents WHERE post_id IN (",
        );
        let mut separated = builder.separated(", ");
        for post_id in post_ids {
            separated.push_bind(post_id);
        }
        separated.push_unseparated(") ORDER BY post_id ASC, sort_order ASC, id ASC");

        builder
            .build_query_as::<PostContentRow>()
            .fetch_all(&self.pool)
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(map_db_error)
    }

    pub async fn find_content_by_id(&self, id: i64) -> Result<Option<PostContentItem>, AppError> {
        sqlx::query_as::<_, PostContentRow>(
            r#"
            SELECT id, post_id, user_id, content_type, content, sort_order, created_at
            FROM post_contents
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
}

#[derive(Debug, FromRow)]
struct PostRow {
    id: i64,
    user_id: i64,
    username: String,
    content: String,
    tags: String,
    comments_count: i64,
    upvote_count: i64,
    collection_count: i64,
    created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct PostContentRow {
    id: i64,
    post_id: i64,
    user_id: i64,
    content_type: i32,
    content: String,
    sort_order: i64,
    created_at: DateTime<Utc>,
}

impl From<PostRow> for PostSummary {
    fn from(row: PostRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            username: row.username,
            content: row.content,
            tags: row.tags,
            comments_count: row.comments_count,
            upvote_count: row.upvote_count,
            collection_count: row.collection_count,
            created_at: row.created_at,
        }
    }
}

impl From<PostContentRow> for PostContentItem {
    fn from(row: PostContentRow) -> Self {
        Self {
            id: row.id,
            post_id: row.post_id,
            user_id: row.user_id,
            content: row.content,
            content_type: row.content_type,
            sort: row.sort_order,
            created_at: row.created_at,
        }
    }
}
