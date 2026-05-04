use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
    time::{Duration, Instant},
};

use anyhow::Context;
use evt_config::Settings;
use evt_domain::{LEGACY_DEFAULT_SPACE_SLUG, PUBLIC_SPACE_SLUG, SiteProfile};
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};

use crate::auth::{JwtService, PasswordService};
use crate::repository::{
    AttachmentRepository, CommentRepository, FollowRepository, FriendshipRepository,
    LegacyPostRepository, MessageRepository, PostRepository, SiteSettingsRepository,
    SpaceRepository, TagRepository, UserProfileRepository, UserRepository, WalletRepository,
};
use crate::storage::LocalAttachmentStorage;

#[derive(Clone)]
pub struct AppContext {
    pub(crate) settings: Arc<Settings>,
    pub(crate) pool: MySqlPool,
    pub(crate) users: UserRepository,
    pub(crate) posts: PostRepository,
    pub(crate) comments: CommentRepository,
    pub(crate) follows: FollowRepository,
    pub(crate) friendships: FriendshipRepository,
    pub(crate) attachments: AttachmentRepository,
    pub(crate) messages: MessageRepository,
    pub(crate) profiles: UserProfileRepository,
    pub(crate) legacy_posts: LegacyPostRepository,
    pub(crate) wallet: WalletRepository,
    pub(crate) site_settings_store: SiteSettingsRepository,
    pub(crate) spaces: SpaceRepository,
    pub(crate) tags: TagRepository,
    pub(crate) attachment_storage: LocalAttachmentStorage,
    pub(crate) site_profile: Arc<RwLock<SiteProfile>>,
    pub(crate) online_metrics: Arc<Mutex<OnlineMetrics>>,
    pub(crate) started_at_unix: i64,
    pub(crate) jwt: JwtService,
    pub(crate) password: PasswordService,
}

#[derive(Default)]
pub(crate) struct OnlineMetrics {
    active_users: HashMap<i64, Instant>,
    history_max_online: usize,
}

impl AppContext {
    pub async fn bootstrap(settings: Settings) -> anyhow::Result<Self> {
        let pool = MySqlPoolOptions::new()
            .max_connections(settings.database.max_connections)
            .connect(&settings.database.url)
            .await
            .with_context(|| "connect mysql")?;

        Self::build(settings, pool).await
    }

    pub async fn bootstrap_lazy(settings: Settings) -> anyhow::Result<Self> {
        let pool = MySqlPoolOptions::new()
            .max_connections(settings.database.max_connections)
            .connect_lazy(&settings.database.url)
            .with_context(|| "create lazy mysql pool")?;

        Self::build(settings, pool).await
    }

    async fn build(settings: Settings, pool: MySqlPool) -> anyhow::Result<Self> {
        let jwt = JwtService::new(
            settings.jwt.secret.clone(),
            settings.jwt.issuer.clone(),
            settings.jwt.expire_seconds,
        );
        let attachment_storage = LocalAttachmentStorage::new(&settings.storage.local_dir).await?;
        let site_profile = Arc::new(RwLock::new(SiteProfile {
            default_space_slug: settings.site.default_space_slug.clone(),
            enable_spaces: settings.site.enable_spaces,
            use_friendship: settings.site.use_friendship,
            enable_trends_bar: settings.site.enable_trends_bar,
            enable_wallet: settings.site.enable_wallet,
            allow_tweet_attachment: settings.site.allow_tweet_attachment,
            allow_tweet_attachment_price: settings.site.allow_tweet_attachment_price,
            allow_tweet_video: settings.site.allow_tweet_video,
            allow_user_register: settings.site.allow_user_register,
            allow_phone_bind: settings.site.allow_phone_bind,
            default_tweet_max_length: settings.site.default_tweet_max_length,
            tweet_web_ellipsis_size: settings.site.tweet_web_ellipsis_size,
            tweet_mobile_ellipsis_size: settings.site.tweet_mobile_ellipsis_size,
            default_tweet_visibility: settings.site.default_tweet_visibility.clone(),
            default_msg_loop_interval: settings.site.default_msg_loop_interval,
            copyright_top: settings.site.copyright_top.clone(),
            copyright_left: settings.site.copyright_left.clone(),
            copyright_left_link: settings.site.copyright_left_link.clone(),
            copyright_right: settings.site.copyright_right.clone(),
            copyright_right_link: settings.site.copyright_right_link.clone(),
        }));

        let app = Self {
            settings: Arc::new(settings),
            users: UserRepository::new(pool.clone()),
            posts: PostRepository::new(pool.clone()),
            comments: CommentRepository::new(pool.clone()),
            follows: FollowRepository::new(pool.clone()),
            friendships: FriendshipRepository::new(pool.clone()),
            attachments: AttachmentRepository::new(pool.clone()),
            messages: MessageRepository::new(pool.clone()),
            profiles: UserProfileRepository::new(pool.clone()),
            legacy_posts: LegacyPostRepository::new(pool.clone()),
            wallet: WalletRepository::new(pool.clone()),
            site_settings_store: SiteSettingsRepository::new(pool.clone()),
            spaces: SpaceRepository::new(pool.clone()),
            tags: TagRepository::new(pool.clone()),
            attachment_storage,
            site_profile,
            online_metrics: Arc::new(Mutex::new(OnlineMetrics::default())),
            started_at_unix: chrono::Utc::now().timestamp(),
            pool,
            jwt,
            password: PasswordService::default(),
        };

        if let Ok(Some(payload)) = app.site_settings_store.load_payload().await {
            app.apply_site_profile_payload(&payload);
        }

        if let Some(owner) = app.users.find_first_summary().await.ok().flatten() {
            let default_space_slug = app.normalized_default_space_slug();
            let _ = app
                .spaces
                .ensure_default_space(&default_space_slug, owner.id)
                .await;
        }

        Ok(app)
    }

    pub fn settings(&self) -> &Settings {
        self.settings.as_ref()
    }

    pub fn site_profile_snapshot(&self) -> SiteProfile {
        self.site_profile
            .read()
            .expect("site profile lock poisoned")
            .clone()
    }

    pub fn apply_site_profile_payload(&self, payload: &serde_json::Value) {
        let mut site = self
            .site_profile
            .write()
            .expect("site profile lock poisoned");
        if let Some(value) = payload
            .get("enable_spaces")
            .and_then(serde_json::Value::as_bool)
        {
            site.enable_spaces = value;
        }
        if let Some(value) = payload
            .get("default_space_slug")
            .and_then(serde_json::Value::as_str)
        {
            site.default_space_slug = if value.trim().is_empty()
                || value.eq_ignore_ascii_case(LEGACY_DEFAULT_SPACE_SLUG)
            {
                PUBLIC_SPACE_SLUG.to_string()
            } else {
                value.to_string()
            };
        }
        if let Some(value) = payload
            .get("use_friendship")
            .and_then(serde_json::Value::as_bool)
        {
            site.use_friendship = value;
        }
        if let Some(value) = payload
            .get("allow_user_register")
            .and_then(serde_json::Value::as_bool)
        {
            site.allow_user_register = value;
        }
        if let Some(value) = payload
            .get("allow_phone_bind")
            .and_then(serde_json::Value::as_bool)
        {
            site.allow_phone_bind = value;
        }
        if let Some(value) = payload
            .get("enable_trends_bar")
            .and_then(serde_json::Value::as_bool)
        {
            site.enable_trends_bar = value;
        }
        if let Some(value) = payload
            .get("enable_wallet")
            .and_then(serde_json::Value::as_bool)
        {
            site.enable_wallet = value;
        }
        if let Some(value) = payload
            .get("allow_tweet_attachment")
            .and_then(serde_json::Value::as_bool)
        {
            site.allow_tweet_attachment = value;
        }
        if let Some(value) = payload
            .get("allow_tweet_attachment_price")
            .and_then(serde_json::Value::as_bool)
        {
            site.allow_tweet_attachment_price = value;
        }
        if let Some(value) = payload
            .get("allow_tweet_video")
            .and_then(serde_json::Value::as_bool)
        {
            site.allow_tweet_video = value;
        }
        if let Some(value) = payload
            .get("default_tweet_max_length")
            .and_then(serde_json::Value::as_u64)
        {
            site.default_tweet_max_length = value as u32;
        }
        if let Some(value) = payload
            .get("tweet_web_ellipsis_size")
            .and_then(serde_json::Value::as_u64)
        {
            site.tweet_web_ellipsis_size = value as u32;
        }
        if let Some(value) = payload
            .get("tweet_mobile_ellipsis_size")
            .and_then(serde_json::Value::as_u64)
        {
            site.tweet_mobile_ellipsis_size = value as u32;
        }
        if let Some(value) = payload
            .get("default_tweet_visibility")
            .and_then(serde_json::Value::as_str)
        {
            site.default_tweet_visibility = value.to_string();
        }
        if let Some(value) = payload
            .get("default_msg_loop_interval")
            .and_then(serde_json::Value::as_u64)
        {
            site.default_msg_loop_interval = value as u32;
        }
        if let Some(value) = payload
            .get("copyright_top")
            .and_then(serde_json::Value::as_str)
        {
            site.copyright_top = value.to_string();
        }
        if let Some(value) = payload
            .get("copyright_left")
            .and_then(serde_json::Value::as_str)
        {
            site.copyright_left = value.to_string();
        }
        if let Some(value) = payload
            .get("copyright_left_link")
            .and_then(serde_json::Value::as_str)
        {
            site.copyright_left_link = value.to_string();
        }
        if let Some(value) = payload
            .get("copyright_right")
            .and_then(serde_json::Value::as_str)
        {
            site.copyright_right = value.to_string();
        }
        if let Some(value) = payload
            .get("copyright_right_link")
            .and_then(serde_json::Value::as_str)
        {
            site.copyright_right_link = value.to_string();
        }
    }

    pub fn mark_online(&self, user_id: i64) {
        let ttl = Duration::from_secs(300);
        let now = Instant::now();
        let mut metrics = self
            .online_metrics
            .lock()
            .expect("online metrics lock poisoned");
        metrics
            .active_users
            .retain(|_, seen_at| now.duration_since(*seen_at) <= ttl);
        metrics.active_users.insert(user_id, now);
        metrics.history_max_online = metrics.history_max_online.max(metrics.active_users.len());
    }

    pub fn online_stats(&self) -> (i32, i32) {
        let ttl = Duration::from_secs(300);
        let now = Instant::now();
        let mut metrics = self
            .online_metrics
            .lock()
            .expect("online metrics lock poisoned");
        metrics
            .active_users
            .retain(|_, seen_at| now.duration_since(*seen_at) <= ttl);
        (
            metrics.active_users.len() as i32,
            metrics.history_max_online as i32,
        )
    }
}
