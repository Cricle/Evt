use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const PUBLIC_SPACE_SLUG: &str = "public";
pub const LEGACY_DEFAULT_SPACE_SLUG: &str = "square";
pub const PUBLIC_SPACE_NAME: &str = "公共广场";
pub const PUBLIC_SPACE_DESCRIPTION: &str = "所有成员默认加入的公共广场";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub phone_number: Option<String>,
    pub password_hash: Option<String>,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    #[default]
    Active,
    Disabled,
}

#[derive(Clone, Debug, Serialize)]
pub struct RegisterResult {
    pub id: i64,
    pub username: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct LoginResult {
    pub token: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct UserProfile {
    pub id: i64,
    pub username: String,
    pub nickname: String,
    pub avatar: String,
    pub is_admin: bool,
    pub is_friend: bool,
    pub is_following: bool,
    pub phone_number: Option<String>,
    pub activation_code: String,
    pub balance: i64,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub posts_count: i64,
    pub comments_count: i64,
    pub followings_count: i64,
    pub followers_count: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct CurrentUser {
    pub id: i64,
    pub username: String,
    pub nickname: String,
    pub avatar: String,
    pub phone_number: Option<String>,
    pub activation_code: String,
    pub balance: i64,
    pub is_admin: bool,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UserSummary {
    pub id: i64,
    pub username: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct UserPreview {
    pub id: i64,
    pub username: String,
    pub nickname: String,
    pub avatar: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UserMeta {
    pub user_id: i64,
    pub nickname: String,
    pub avatar: String,
    pub activation_code: String,
    pub is_admin: bool,
    pub balance: i64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpaceRole {
    Member = 0,
    Admin = 1,
    Owner = 2,
}

impl SpaceRole {
    pub fn can_manage_members(self) -> bool {
        matches!(self, Self::Admin | Self::Owner)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpaceVisibility {
    Public = 0,
    Private = 1,
}

#[derive(Clone, Debug, Serialize)]
pub struct SpaceSummary {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub owner_user_id: i64,
    pub visibility: SpaceVisibility,
    pub members_count: i64,
    pub current_user_role: Option<SpaceRole>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SpaceMemberSummary {
    pub space_id: i64,
    pub user_id: i64,
    pub username: String,
    pub nickname: String,
    pub avatar: String,
    pub role: SpaceRole,
    pub invited_by_user_id: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FollowActionResult {
    pub following: bool,
    pub user: UserSummary,
}

#[derive(Clone, Debug, Serialize)]
pub struct AttachmentSummary {
    pub id: i64,
    pub user_id: i64,
    pub file_name: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct AttachmentDownload {
    pub file_name: String,
    pub content_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Serialize)]
pub struct MessageSummary {
    pub id: i64,
    pub sender_user_id: i64,
    pub sender_username: String,
    pub receiver_user_id: i64,
    pub receiver_username: String,
    pub content: String,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct LegacyMessageSummary {
    pub id: i64,
    pub sender_user_id: i64,
    pub receiver_user_id: i64,
    pub message_type: i32,
    pub brief: String,
    pub content: String,
    pub post_id: i64,
    pub comment_id: i64,
    pub reply_id: i64,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UnreadCount {
    pub unread_count: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct PostSummary {
    pub id: i64,
    pub space_id: i64,
    pub user_id: i64,
    pub username: String,
    pub content: String,
    pub tags: String,
    pub comments_count: i64,
    pub upvote_count: i64,
    pub collection_count: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct LegacyPostState {
    pub post_id: i64,
    pub attachment_price: i64,
    pub visibility: i32,
    pub is_lock: bool,
    pub is_top: bool,
    pub is_essence: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct TagSummary {
    pub id: i64,
    pub space_id: i64,
    pub user_id: i64,
    pub username: String,
    pub tag: String,
    pub quote_num: i64,
    pub created_at: DateTime<Utc>,
    pub is_following: bool,
    pub is_top: bool,
    pub is_pin: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommentSummary {
    pub id: i64,
    pub post_id: i64,
    pub user_id: i64,
    pub username: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct LegacyCommentState {
    pub comment_id: i64,
    pub is_essence: bool,
    pub is_reaction: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommentReplySummary {
    pub id: i64,
    pub comment_id: i64,
    pub user_id: i64,
    pub at_user_id: i64,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct CreateContentInput {
    pub content: String,
    pub content_type: i32,
    pub sort: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct PostContentItem {
    pub id: i64,
    pub post_id: i64,
    pub user_id: i64,
    pub content: String,
    pub content_type: i32,
    pub sort: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommentContentItem {
    pub id: i64,
    pub comment_id: i64,
    pub user_id: i64,
    pub content: String,
    pub content_type: i32,
    pub sort: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PostReactionSummary {
    pub emoji: String,
    pub count: i64,
    pub active: bool,
    pub users: Vec<UserPreview>,
    pub comment_ids: Vec<i64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TogglePostReactionResult {
    pub active: bool,
    pub reactions: Vec<PostReactionSummary>,
    pub comment_count: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct PagedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct VersionInfo {
    pub name: String,
    pub version: String,
    pub environment: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct SiteProfile {
    pub default_space_slug: String,
    pub enable_spaces: bool,
    pub use_friendship: bool,
    pub enable_trends_bar: bool,
    pub enable_wallet: bool,
    pub allow_tweet_attachment: bool,
    pub allow_tweet_attachment_price: bool,
    pub allow_tweet_video: bool,
    pub allow_user_register: bool,
    pub allow_phone_bind: bool,
    pub default_tweet_max_length: u32,
    pub tweet_web_ellipsis_size: u32,
    pub tweet_mobile_ellipsis_size: u32,
    pub default_tweet_visibility: String,
    pub default_msg_loop_interval: u32,
    pub copyright_top: String,
    pub copyright_left: String,
    pub copyright_left_link: String,
    pub copyright_right: String,
    pub copyright_right_link: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct SiteSettingSchemaItem {
    pub key: String,
    pub group: String,
    pub section: String,
    pub value_type: String,
    pub label: String,
    pub description: String,
    pub apply_mode: String,
    pub secret: bool,
    pub readonly: bool,
    pub active: bool,
    pub bootstrap_value: serde_json::Value,
    pub options: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SiteSettingValueItem {
    pub key: String,
    pub value: serde_json::Value,
    pub effective_value: serde_json::Value,
    pub source: String,
    pub pending_restart: bool,
    pub configured: bool,
    pub active: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub uid: i64,
    pub username: String,
    pub iss: String,
    pub exp: usize,
}

#[derive(Clone, Debug)]
pub struct UserIdentity {
    pub id: i64,
    pub username: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct WalletRechargeSummary {
    pub id: i64,
    pub user_id: i64,
    pub amount: i64,
    pub trade_no: String,
    pub trade_status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct WalletStatementSummary {
    pub id: i64,
    pub user_id: i64,
    pub change_amount: i64,
    pub balance_snapshot: i64,
    pub reason: String,
    pub post_id: i64,
    pub created_at: DateTime<Utc>,
}
