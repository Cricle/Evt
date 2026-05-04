use evt_domain::{AppError, SiteSettingSchemaItem, SiteSettingValueItem};
use serde_json::{Value, json};

use crate::AppContext;

type SettingCatalogItem = (&'static str, &'static str, &'static str, &'static str, &'static str, &'static str);

const EDITABLE_KEYS: &[SettingCatalogItem] = &[
    (
        "enable_spaces",
        "web",
        "spaces",
        "bool",
        "启用广场体系",
        "开启后，站点会按广场隔离内容、成员和权限；关闭后，前端仍会回退到默认公共广场视角。",
    ),
    (
        "default_space_slug",
        "web",
        "spaces",
        "string",
        "默认广场标识",
        "指定新用户默认进入、系统兜底使用的广场 slug。建议始终指向一个真实存在且可访问的公共广场。",
    ),
    (
        "allow_user_register",
        "web",
        "accounts",
        "bool",
        "允许新用户注册",
        "关闭后，未登录访客将不能自行创建账号，只能由已有账号登录或由管理员预置用户。",
    ),
    (
        "allow_phone_bind",
        "web",
        "accounts",
        "bool",
        "允许绑定手机号",
        "控制前端是否展示手机号绑定入口，以及用户是否可以通过短信验证码完成手机号绑定。",
    ),
    (
        "use_friendship",
        "web",
        "social",
        "bool",
        "启用好友关系",
        "开启后，会显示好友、好友申请、好友可见等社交能力；关闭后，相关入口和交互会一并隐藏。",
    ),
    (
        "enable_trends_bar",
        "web",
        "social",
        "bool",
        "启用趋势栏",
        "控制首页是否展示趋势联系人/快捷切换栏。适合强调活跃联系人和关注动态的场景。",
    ),
    (
        "enable_wallet",
        "web",
        "payments",
        "bool",
        "启用钱包功能",
        "开启后，用户将看到钱包入口以及与余额、支付相关的前端能力；关闭后，钱包模块整体隐藏。",
    ),
    (
        "allow_tweet_attachment",
        "web",
        "publishing",
        "bool",
        "允许发布附件",
        "控制编辑器里是否允许上传并附带文件附件，例如压缩包、文档或二进制文件。",
    ),
    (
        "allow_tweet_attachment_price",
        "web",
        "payments",
        "bool",
        "允许附件定价",
        "开启后，发布者可以为附件设置价格；关闭后，附件只能以免费形式发布。",
    ),
    (
        "allow_tweet_video",
        "web",
        "publishing",
        "bool",
        "允许发布视频",
        "控制编辑器中视频上传能力以及服务端对视频附件上传的放行逻辑。",
    ),
    (
        "default_tweet_max_length",
        "web",
        "publishing",
        "int",
        "动态最大字数",
        "限制一条动态允许输入的最大纯文本长度。前端编辑器和服务端校验都会使用这个值。",
    ),
    (
        "tweet_web_ellipsis_size",
        "web",
        "reading",
        "int",
        "Web 摘要折叠长度",
        "控制桌面端信息流里，一条动态在未展开时最多展示多少文本，超过后会折叠显示。",
    ),
    (
        "tweet_mobile_ellipsis_size",
        "web",
        "reading",
        "int",
        "移动端摘要折叠长度",
        "控制移动端信息流里，一条动态在未展开时最多展示多少文本，超过后会折叠显示。",
    ),
    (
        "default_tweet_visibility",
        "web",
        "publishing",
        "string",
        "默认可见范围",
        "指定发布动态时默认选中的可见性，例如公开、关注可见、好友可见或私密。",
    ),
    (
        "default_msg_loop_interval",
        "web",
        "reading",
        "int",
        "消息轮询间隔",
        "前端轮询未读消息的时间间隔，单位为毫秒。值越小越实时，但请求频率也会更高。",
    ),
    (
        "copyright_top",
        "web",
        "branding",
        "string",
        "页脚主文案",
        "显示在站点页脚顶部的主版权或品牌文字，通常用于站点名、年份或备案信息。",
    ),
    (
        "copyright_left",
        "web",
        "branding",
        "string",
        "页脚左侧文案",
        "显示在页脚左侧链接位置的文字。留空则不显示该链接。",
    ),
    (
        "copyright_left_link",
        "web",
        "branding",
        "string",
        "页脚左侧链接",
        "为页脚左侧文案配置跳转地址。通常配合页脚左侧文案一起使用。",
    ),
    (
        "copyright_right",
        "web",
        "branding",
        "string",
        "页脚右侧文案",
        "显示在页脚右侧链接位置的文字。留空则不显示该链接。",
    ),
    (
        "copyright_right_link",
        "web",
        "branding",
        "string",
        "页脚右侧链接",
        "为页脚右侧文案配置跳转地址。通常用于项目主页、文档或开源仓库。",
    ),
];

impl AppContext {
    pub async fn admin_settings_schema(&self) -> Result<Vec<SiteSettingSchemaItem>, AppError> {
        let current = self.site_profile_snapshot();
        let mut items = Vec::with_capacity(EDITABLE_KEYS.len());
        for (key, group, section, value_type, label, description) in EDITABLE_KEYS {
            let bootstrap_value = match *key {
                "enable_spaces" => json!(current.enable_spaces),
                "default_space_slug" => json!(current.default_space_slug),
                "allow_user_register" => json!(current.allow_user_register),
                "allow_phone_bind" => json!(current.allow_phone_bind),
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
                description: (*description).to_string(),
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
        for (key, _, _, _, _, _) in EDITABLE_KEYS {
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
        "enable_spaces" => json!(profile.enable_spaces),
        "default_space_slug" => json!(profile.default_space_slug),
        "allow_user_register" => json!(profile.allow_user_register),
        "allow_phone_bind" => json!(profile.allow_phone_bind),
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
