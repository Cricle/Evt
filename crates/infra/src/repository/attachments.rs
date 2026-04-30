use chrono::{DateTime, Utc};
use paopao_domain::{AppError, AttachmentSummary};
use sqlx::{FromRow, MySqlPool};

use super::map_db_error;

#[derive(Clone)]
pub struct AttachmentRepository {
    pool: MySqlPool,
}

impl AttachmentRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        user_id: i64,
        file_name: &str,
        content_type: &str,
        size_bytes: i64,
        storage_key: &str,
    ) -> Result<AttachmentSummary, AppError> {
        let result = sqlx::query(
            r#"
            INSERT INTO attachments (user_id, file_name, content_type, size_bytes, storage_key)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(user_id)
        .bind(file_name)
        .bind(content_type)
        .bind(size_bytes)
        .bind(storage_key)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        self.find_by_id(result.last_insert_id() as i64)
            .await?
            .map(|record| record.summary)
            .ok_or_else(|| AppError::Internal("created attachment cannot be loaded".into()))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<AttachmentRecord>, AppError> {
        sqlx::query_as::<_, AttachmentRow>(
            r#"
            SELECT id, user_id, file_name, content_type, size_bytes, storage_key, created_at
            FROM attachments
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
struct AttachmentRow {
    id: i64,
    user_id: i64,
    file_name: String,
    content_type: String,
    size_bytes: i64,
    storage_key: String,
    created_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct AttachmentRecord {
    pub summary: AttachmentSummary,
    pub storage_key: String,
}

impl From<AttachmentRow> for AttachmentRecord {
    fn from(row: AttachmentRow) -> Self {
        Self {
            summary: AttachmentSummary {
                id: row.id,
                user_id: row.user_id,
                file_name: row.file_name,
                content_type: row.content_type,
                size_bytes: row.size_bytes,
                created_at: row.created_at,
            },
            storage_key: row.storage_key,
        }
    }
}
