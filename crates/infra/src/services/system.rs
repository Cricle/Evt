use chrono::Utc;
use evt_domain::{
    AppError, LEGACY_DEFAULT_SPACE_SLUG, PUBLIC_SPACE_SLUG, SiteProfile, VersionInfo,
};

use crate::AppContext;

impl AppContext {
    pub fn version(&self) -> VersionInfo {
        VersionInfo {
            name: self.settings.app.name.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: self.settings.app.env.clone(),
        }
    }

    pub fn site_profile(&self) -> SiteProfile {
        let mut profile = self.site_profile_snapshot();
        if profile.default_space_slug.trim().is_empty()
            || profile
                .default_space_slug
                .eq_ignore_ascii_case(LEGACY_DEFAULT_SPACE_SLUG)
        {
            profile.default_space_slug = PUBLIC_SPACE_SLUG.to_string();
        }
        profile
    }

    pub async fn healthcheck(&self) -> Result<(), AppError> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|err| AppError::Internal(format!("database healthcheck failed: {err}")))
    }

    pub async fn register_user_count(&self) -> Result<i64, AppError> {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await
            .map_err(|err| AppError::Internal(format!("database operation failed: {err}")))
    }

    pub fn admin_site_status_snapshot(&self) -> (i32, i32, i64) {
        let (online_user_count, history_max_online) = self.online_stats();
        (online_user_count, history_max_online, self.started_at_unix)
    }

    pub fn now(&self) -> chrono::DateTime<Utc> {
        Utc::now()
    }
}
