mod attachments;
mod comments;
mod follows;
pub(crate) mod friendships;
pub(crate) mod legacy_posts;
mod messages;
mod posts;
mod profiles;
mod site_settings;
mod tags;
mod users;
mod wallet;

pub use attachments::AttachmentRepository;
pub use comments::CommentRepository;
pub use follows::FollowRepository;
pub use friendships::FriendshipRepository;
pub use legacy_posts::LegacyPostRepository;
pub use messages::MessageRepository;
pub use posts::PostRepository;
pub use profiles::UserProfileRepository;
pub use site_settings::SiteSettingsRepository;
pub use tags::TagRepository;
pub use users::UserRepository;
pub use wallet::WalletRepository;

use paopao_domain::AppError;

pub(crate) fn map_db_error(err: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_err) = &err {
        if db_err.is_unique_violation() {
            return AppError::Conflict("resource already exists".into());
        }
    }
    AppError::Internal(format!("database operation failed: {err}"))
}
