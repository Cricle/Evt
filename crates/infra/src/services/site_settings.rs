use evt_domain::{AppError, SiteSettingSchemaItem, SiteSettingValueItem};
use serde_json::{Value, json};

use crate::AppContext;

const EDITABLE_KEYS: &[(&str, &str, &str, &str, &str)] = &[
    ("use_friendship", "web", "profile", "bool", "Use friendship"),
    (
        "enable_trends_bar",
        "web",
        "profile",
        "bool",
        "Enable trends bar",
    ),
    ("enable_wallet", "web", "profile", "bool", "Enable wallet"),
    (
        "allow_tweet_attachment",
        "web",
        "profile",
        "bool",
        "Allow attachments",
    ),
    (
        "allow_tweet_attachment_price",
        "web",
        "profile",
        "bool",
        "Allow paid attachments",
    ),
    (
        "allow_tweet_video",
        "web",
        "profile",
        "bool",
        "Allow video posts",
    ),
    (
        "default_tweet_max_length",
        "web",
        "profile",
        "int",
        "Default tweet max length",
    ),
    (
        "tweet_web_ellipsis_size",
        "web",
        "profile",
        "int",
        "Web ellipsis size",
    ),
    (
        "tweet_mobile_ellipsis_size",
        "web",
        "profile",
        "int",
        "Mobile ellipsis size",
    ),
    (
        "default_tweet_visibility",
        "web",
        "profile",
        "string",
        "Default tweet visibility",
    ),
    (
        "default_msg_loop_interval",
        "web",
        "profile",
        "int",
        "Message polling interval",
    ),
    ("copyright_top", "web", "profile", "string", "Copyright top"),
    (
        "copyright_left",
        "web",
        "profile",
        "string",
        "Copyright left",
    ),
    (
        "copyright_left_link",
        "web",
        "profile",
        "string",
        "Copyright left link",
    ),
    (
        "copyright_right",
        "web",
        "profile",
        "string",
        "Copyright right",
    ),
    (
        "copyright_right_link",
        "web",
        "profile",
        "string",
        "Copyright right link",
    ),
];

impl AppContext {
    pub async fn admin_settings_schema(&self) -> Result<Vec<SiteSettingSchemaItem>, AppError> {
        let current = self.site_profile_snapshot();
        let mut items = Vec::with_capacity(EDITABLE_KEYS.len());
        for (key, group, section, value_type, label) in EDITABLE_KEYS {
            let bootstrap_value = match *key {
                "use_friendship" => json!(current.use_friendship),
                "enable_trends_bar" => json!(current.enable_trends_bar),
                "enable_wallet" => json!(current.enable_wallet),
                "allow_tweet_attachment" => json!(current.allow_tweet_attachment),
                "allow_tweet_attachment_price" => json!(current.allow_tweet_attachment_price),
                "allow_tweet_video" => json!(current.allow_tweet_video),
                "default_tweet_max_length" => json!(current.default_tweet_max_length),
                "tweet_web_ellipsis_size" => json!(current.tweet_web_ellipsis_size),
                "tweet_mobile_ellipsis_size" => json!(current.tweet_mobile_ellipsis_size),
                "default_tweet_visibility" => json!(current.default_tweet_visibility),
                "default_msg_loop_interval" => json!(current.default_msg_loop_interval),
                "copyright_top" => json!(current.copyright_top),
                "copyright_left" => json!(current.copyright_left),
                "copyright_left_link" => json!(current.copyright_left_link),
                "copyright_right" => json!(current.copyright_right),
                "copyright_right_link" => json!(current.copyright_right_link),
                _ => Value::Null,
            };
            items.push(SiteSettingSchemaItem {
                key: format!("web_profile.{key}"),
                group: (*group).to_string(),
                section: (*section).to_string(),
                value_type: (*value_type).to_string(),
                label: (*label).to_string(),
                description: (*label).to_string(),
                apply_mode: "live".to_string(),
                secret: false,
                readonly: false,
                active: true,
                bootstrap_value,
                options: if *key == "default_tweet_visibility" {
                    Some(json!([
                        { "label": "public", "value": "public" },
                        { "label": "following", "value": "following" },
                        { "label": "friend", "value": "friend" },
                        { "label": "private", "value": "private" }
                    ]))
                } else {
                    None
                },
            });
        }
        Ok(items)
    }

    pub async fn admin_settings_values(&self) -> Result<Vec<SiteSettingValueItem>, AppError> {
        let current = self.site_profile_snapshot();
        let payload = self
            .site_settings_store
            .load_payload()
            .await?
            .unwrap_or_else(|| json!({}));
        let mut items = Vec::with_capacity(EDITABLE_KEYS.len());
        for (key, _, _, _, _) in EDITABLE_KEYS {
            let value = payload
                .get(*key)
                .cloned()
                .unwrap_or_else(|| setting_value_from_profile(&current, key));
            items.push(SiteSettingValueItem {
                key: format!("web_profile.{key}"),
                value: value.clone(),
                effective_value: value,
                source: if payload.get(*key).is_some() {
                    "override".to_string()
                } else {
                    "bootstrap".to_string()
                },
                pending_restart: false,
                configured: true,
                active: true,
            });
        }
        Ok(items)
    }

    pub async fn save_admin_settings(
        &self,
        items: &[(String, Value)],
    ) -> Result<Vec<SiteSettingValueItem>, AppError> {
        let mut payload = self
            .site_settings_store
            .load_payload()
            .await?
            .unwrap_or_else(|| json!({}));
        for (key, value) in items {
            let trimmed = key
                .strip_prefix("web_profile.")
                .ok_or_else(|| AppError::Validation("unknown setting key".into()))?;
            payload[trimmed] = value.clone();
        }
        self.site_settings_store.save_payload(&payload).await?;
        self.apply_site_profile_payload(&payload);
        self.admin_settings_values().await
    }
}

fn setting_value_from_profile(profile: &evt_domain::SiteProfile, key: &str) -> Value {
    match key {
        "use_friendship" => json!(profile.use_friendship),
        "enable_trends_bar" => json!(profile.enable_trends_bar),
        "enable_wallet" => json!(profile.enable_wallet),
        "allow_tweet_attachment" => json!(profile.allow_tweet_attachment),
        "allow_tweet_attachment_price" => json!(profile.allow_tweet_attachment_price),
        "allow_tweet_video" => json!(profile.allow_tweet_video),
        "default_tweet_max_length" => json!(profile.default_tweet_max_length),
        "tweet_web_ellipsis_size" => json!(profile.tweet_web_ellipsis_size),
        "tweet_mobile_ellipsis_size" => json!(profile.tweet_mobile_ellipsis_size),
        "default_tweet_visibility" => json!(profile.default_tweet_visibility),
        "default_msg_loop_interval" => json!(profile.default_msg_loop_interval),
        "copyright_top" => json!(profile.copyright_top),
        "copyright_left" => json!(profile.copyright_left),
        "copyright_left_link" => json!(profile.copyright_left_link),
        "copyright_right" => json!(profile.copyright_right),
        "copyright_right_link" => json!(profile.copyright_right_link),
        _ => Value::Null,
    }
}
