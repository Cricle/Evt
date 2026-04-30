use paopao_domain::AppError;
use serde_json::Value;
use sqlx::MySqlPool;

use super::map_db_error;

#[derive(Clone)]
pub struct SiteSettingsRepository {
    pool: MySqlPool,
}

impl SiteSettingsRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn load_payload(&self) -> Result<Option<Value>, AppError> {
        sqlx::query_scalar::<_, Value>("SELECT payload FROM site_settings WHERE id = 1 LIMIT 1")
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    pub async fn save_payload(&self, payload: &Value) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO site_settings (id, payload)
            VALUES (1, ?)
            ON DUPLICATE KEY UPDATE
              payload = VALUES(payload),
              updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(payload)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }
}
