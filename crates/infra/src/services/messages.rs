use paopao_domain::{
    AppError, LegacyMessageSummary, MessageSummary, PagedResponse, UnreadCount, UserIdentity,
};

use crate::AppContext;

impl AppContext {
    pub async fn list_messages(
        &self,
        actor: &UserIdentity,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<MessageSummary>, AppError> {
        self.messages
            .list_for_receiver(actor.id, page, page_size)
            .await
    }

    pub async fn unread_message_count(
        &self,
        actor: &UserIdentity,
    ) -> Result<UnreadCount, AppError> {
        let unread_count = self.messages.unread_count(actor.id).await?;
        Ok(UnreadCount { unread_count })
    }

    pub async fn send_message(
        &self,
        actor: &UserIdentity,
        receiver_username: &str,
        content: &str,
    ) -> Result<MessageSummary, AppError> {
        let receiver = self
            .users
            .find_summary_by_username(receiver_username)
            .await?
            .ok_or_else(|| AppError::NotFound("receiver user not found".into()))?;

        if actor.id == receiver.id {
            return Err(AppError::Validation(
                "cannot send message to yourself".into(),
            ));
        }

        let content = content.trim();
        if content.is_empty() {
            return Err(AppError::Validation(
                "message content cannot be empty".into(),
            ));
        }

        self.messages.create(actor.id, receiver.id, content).await
    }

    pub async fn mark_message_read(
        &self,
        actor: &UserIdentity,
        message_id: i64,
    ) -> Result<(), AppError> {
        let message = self
            .messages
            .find_by_id(message_id)
            .await?
            .ok_or_else(|| AppError::NotFound("message not found".into()))?;

        if message.receiver_user_id != actor.id {
            return Err(AppError::Unauthorized(
                "cannot mark another user's message as read".into(),
            ));
        }

        self.messages.mark_read(actor.id, message_id).await
    }

    pub async fn mark_all_messages_read(&self, actor: &UserIdentity) -> Result<(), AppError> {
        self.messages.mark_all_read(actor.id).await
    }

    pub async fn list_legacy_messages(
        &self,
        actor: &UserIdentity,
        style: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<LegacyMessageSummary>, AppError> {
        self.messages
            .list_legacy(actor.id, style, page, page_size)
            .await
    }

    pub async fn unread_legacy_message_count(&self, actor: &UserIdentity) -> Result<i64, AppError> {
        self.messages.unread_legacy_count(actor.id).await
    }

    pub async fn send_legacy_message(
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
        self.messages
            .create_legacy(
                sender_user_id,
                receiver_user_id,
                message_type,
                brief,
                content,
                post_id,
                comment_id,
                reply_id,
            )
            .await
    }
}
