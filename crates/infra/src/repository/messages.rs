use chrono::{DateTime, Utc};
use evt_domain::{AppError, LegacyMessageSummary, MessageSummary, PagedResponse};
use sqlx::{FromRow, MySqlPool};

use super::map_db_error;

#[derive(Clone)]
pub struct MessageRepository {
    pool: MySqlPool,
}

impl MessageRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        sender_user_id: i64,
        receiver_user_id: i64,
        content: &str,
    ) -> Result<MessageSummary, AppError> {
        self.create_legacy(
            sender_user_id,
            receiver_user_id,
            4,
            "给你发送新私信了",
            content,
            0,
            0,
            0,
        )
        .await?;

        self.find_latest_whisper(sender_user_id, receiver_user_id)
            .await
    }

    pub async fn create_legacy(
        &self,
        sender_user_id: i64,
        receiver_user_id: i64,
        message_type: i32,
        brief: &str,
        content: &str,
        post_id: i64,
        comment_id: i64,
        reply_id: i64,
    ) -> Result<LegacyMessageSummary, AppError> {
        let result = sqlx::query(
            r#"
            INSERT INTO messages (sender_user_id, receiver_user_id, type, brief, content, post_id, comment_id, reply_id, is_read)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(sender_user_id)
        .bind(receiver_user_id)
        .bind(message_type)
        .bind(brief)
        .bind(content)
        .bind(post_id)
        .bind(comment_id)
        .bind(reply_id)
        .bind(false)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        self.find_legacy_by_id(result.last_insert_id() as i64)
            .await?
            .ok_or_else(|| AppError::Internal("created message cannot be loaded".into()))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<MessageSummary>, AppError> {
        sqlx::query_as::<_, MessageRow>(
            r#"
            SELECT
              m.id,
              m.sender_user_id,
              sender.username AS sender_username,
              m.receiver_user_id,
              receiver.username AS receiver_username,
              m.content,
              m.is_read,
              m.created_at
            FROM messages m
            INNER JOIN users sender ON sender.id = m.sender_user_id
            INNER JOIN users receiver ON receiver.id = m.receiver_user_id
            WHERE m.id = ?
            LIMIT 1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Into::into))
        .map_err(map_db_error)
    }

    async fn find_latest_whisper(
        &self,
        sender_user_id: i64,
        receiver_user_id: i64,
    ) -> Result<MessageSummary, AppError> {
        sqlx::query_as::<_, MessageRow>(
            r#"
            SELECT
              m.id,
              m.sender_user_id,
              sender.username AS sender_username,
              m.receiver_user_id,
              receiver.username AS receiver_username,
              m.content,
              m.is_read,
              m.created_at
            FROM messages m
            INNER JOIN users sender ON sender.id = m.sender_user_id
            INNER JOIN users receiver ON receiver.id = m.receiver_user_id
            WHERE m.sender_user_id = ? AND m.receiver_user_id = ? AND m.type = 4
            ORDER BY m.id DESC
            LIMIT 1
            "#,
        )
        .bind(sender_user_id)
        .bind(receiver_user_id)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(map_db_error)
    }

    pub async fn find_legacy_by_id(
        &self,
        id: i64,
    ) -> Result<Option<LegacyMessageSummary>, AppError> {
        sqlx::query_as::<_, LegacyMessageRow>(
            r#"
            SELECT id, sender_user_id, receiver_user_id, type, brief, content, post_id, comment_id, reply_id, is_read, created_at
            FROM messages
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

    pub async fn list_for_receiver(
        &self,
        receiver_user_id: i64,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<MessageSummary>, AppError> {
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let page_size_i64 = page_size as i64;

        let total = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM messages
            WHERE receiver_user_id = ?
            "#,
        )
        .bind(receiver_user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        let items = sqlx::query_as::<_, MessageRow>(
            r#"
            SELECT
              m.id,
              m.sender_user_id,
              sender.username AS sender_username,
              m.receiver_user_id,
              receiver.username AS receiver_username,
              m.content,
              m.is_read,
              m.created_at
            FROM messages m
            INNER JOIN users sender ON sender.id = m.sender_user_id
            INNER JOIN users receiver ON receiver.id = m.receiver_user_id
            WHERE m.receiver_user_id = ?
            ORDER BY m.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(receiver_user_id)
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

    pub async fn unread_count(&self, receiver_user_id: i64) -> Result<i64, AppError> {
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM messages
            WHERE receiver_user_id = ? AND is_read = ?
            "#,
        )
        .bind(receiver_user_id)
        .bind(false)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    pub async fn unread_legacy_count(&self, receiver_user_id: i64) -> Result<i64, AppError> {
        self.unread_count(receiver_user_id).await
    }

    pub async fn list_legacy(
        &self,
        user_id: i64,
        style: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<LegacyMessageSummary>, AppError> {
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let limit = page_size as i64;
        let (where_sql, binds) = legacy_where_clause(user_id, style);

        let total_sql = format!("SELECT COUNT(*) FROM messages m WHERE {where_sql}");
        let mut total_query = sqlx::query_scalar::<_, i64>(&total_sql);
        for bind in &binds {
            total_query = total_query.bind(*bind);
        }
        let total = total_query
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;

        let list_sql = format!(
            "SELECT id, sender_user_id, receiver_user_id, type, brief, content, post_id, comment_id, reply_id, is_read, created_at FROM messages m WHERE {where_sql} ORDER BY id DESC LIMIT ? OFFSET ?"
        );
        let mut list_query = sqlx::query_as::<_, LegacyMessageRow>(&list_sql);
        for bind in &binds {
            list_query = list_query.bind(*bind);
        }
        let items = list_query
            .bind(limit)
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

    pub async fn mark_read(&self, receiver_user_id: i64, message_id: i64) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE messages
            SET is_read = ?
            WHERE id = ? AND receiver_user_id = ?
            "#,
        )
        .bind(true)
        .bind(message_id)
        .bind(receiver_user_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn mark_all_read(&self, receiver_user_id: i64) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE messages
            SET is_read = ?
            WHERE receiver_user_id = ?
            "#,
        )
        .bind(true)
        .bind(receiver_user_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }
}

#[derive(Debug, FromRow)]
struct MessageRow {
    id: i64,
    sender_user_id: i64,
    sender_username: String,
    receiver_user_id: i64,
    receiver_username: String,
    content: String,
    is_read: bool,
    created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct LegacyMessageRow {
    id: i64,
    sender_user_id: i64,
    receiver_user_id: i64,
    r#type: i32,
    brief: String,
    content: String,
    post_id: i64,
    comment_id: i64,
    reply_id: i64,
    is_read: bool,
    created_at: DateTime<Utc>,
}

impl From<MessageRow> for MessageSummary {
    fn from(row: MessageRow) -> Self {
        Self {
            id: row.id,
            sender_user_id: row.sender_user_id,
            sender_username: row.sender_username,
            receiver_user_id: row.receiver_user_id,
            receiver_username: row.receiver_username,
            content: row.content,
            is_read: row.is_read,
            created_at: row.created_at,
        }
    }
}

impl From<LegacyMessageRow> for LegacyMessageSummary {
    fn from(row: LegacyMessageRow) -> Self {
        Self {
            id: row.id,
            sender_user_id: row.sender_user_id,
            receiver_user_id: row.receiver_user_id,
            message_type: row.r#type,
            brief: row.brief,
            content: row.content,
            post_id: row.post_id,
            comment_id: row.comment_id,
            reply_id: row.reply_id,
            is_read: row.is_read,
            created_at: row.created_at,
        }
    }
}

fn legacy_where_clause(user_id: i64, style: &str) -> (&'static str, Vec<i64>) {
    match style {
        "system" => (
            "m.receiver_user_id = ? AND m.type IN (1, 2, 3, 99)",
            vec![user_id],
        ),
        "whisper" => (
            "(m.receiver_user_id = ? OR m.sender_user_id = ?) AND m.type = 4",
            vec![user_id, user_id],
        ),
        "requesting" => ("m.receiver_user_id = ? AND m.type = 5", vec![user_id]),
        "unread" => (
            "m.receiver_user_id = ? AND m.is_read = FALSE",
            vec![user_id],
        ),
        _ => (
            "m.receiver_user_id = ? OR (m.sender_user_id = ? AND m.type = 4)",
            vec![user_id, user_id],
        ),
    }
}
