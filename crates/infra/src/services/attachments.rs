use paopao_domain::{AppError, AttachmentDownload, AttachmentSummary, UserIdentity};

use crate::AppContext;

const LEGACY_ATTACHMENT_INCOME_RATE: f64 = 0.8;

impl AppContext {
    pub async fn upload_attachment(
        &self,
        actor: &UserIdentity,
        file_name: &str,
        content_type: Option<&str>,
        bytes: &[u8],
    ) -> Result<AttachmentSummary, AppError> {
        if !self.site_profile_snapshot().allow_tweet_attachment {
            return Err(AppError::Validation("attachments are disabled".into()));
        }
        if bytes.is_empty() {
            return Err(AppError::Validation("attachment file is empty".into()));
        }

        let file_name = file_name.trim();
        if file_name.is_empty() {
            return Err(AppError::Validation(
                "attachment file name is required".into(),
            ));
        }

        let storage_key = self.attachment_storage.save(file_name, bytes).await?;

        self.attachments
            .create(
                actor.id,
                file_name,
                content_type.unwrap_or("application/octet-stream"),
                bytes.len() as i64,
                &storage_key,
            )
            .await
    }

    pub async fn download_attachment(
        &self,
        attachment_id: i64,
    ) -> Result<AttachmentDownload, AppError> {
        let attachment = self
            .attachments
            .find_by_id(attachment_id)
            .await?
            .ok_or_else(|| AppError::NotFound("attachment not found".into()))?;
        let bytes = self
            .attachment_storage
            .read(&attachment.storage_key)
            .await?;

        Ok(AttachmentDownload {
            file_name: attachment.summary.file_name,
            content_type: attachment.summary.content_type,
            bytes,
        })
    }

    pub async fn attachment_download_precheck(
        &self,
        actor: &UserIdentity,
        content_id: i64,
    ) -> Result<bool, AppError> {
        let content = self
            .posts
            .find_content_by_id(content_id)
            .await?
            .ok_or_else(|| AppError::NotFound("attachment content not found".into()))?;
        let post = self.get_post(content.post_id).await?;
        let current = self.get_current_user(actor).await?;

        if content.content_type != 8 {
            return Ok(true);
        }
        if post.user_id == actor.id || current.is_admin {
            return Ok(true);
        }

        self.wallet.has_attachment_purchase(actor.id, post.id).await
    }

    pub async fn download_attachment_from_content(
        &self,
        actor: &UserIdentity,
        content_id: i64,
    ) -> Result<AttachmentDownload, AppError> {
        let attachment_id = self
            .resolve_attachment_id_from_content(actor, content_id)
            .await?;
        self.download_attachment(attachment_id).await
    }

    pub async fn resolve_attachment_id_from_content(
        &self,
        actor: &UserIdentity,
        content_id: i64,
    ) -> Result<i64, AppError> {
        let content = self
            .posts
            .find_content_by_id(content_id)
            .await?
            .ok_or_else(|| AppError::NotFound("attachment content not found".into()))?;
        let post = self.get_post(content.post_id).await?;
        let attachment_id = parse_attachment_id(&content.content)
            .ok_or_else(|| AppError::Validation("invalid attachment content path".into()))?;
        let current = self.get_current_user(actor).await?;

        if content.content_type == 8 && post.user_id != actor.id && !current.is_admin {
            let purchased = self
                .wallet
                .has_attachment_purchase(actor.id, post.id)
                .await?;
            if !purchased {
                self.profiles
                    .ensure_defaults(actor.id, &actor.username)
                    .await?;
                let author = self
                    .users
                    .find_by_id(post.user_id)
                    .await?
                    .ok_or_else(|| AppError::NotFound("post author not found".into()))?;
                self.profiles
                    .ensure_defaults(author.id, &author.username)
                    .await?;
                let price = post_attachment_price(post.id, self).await?;
                let author_income = ((price as f64) * LEGACY_ATTACHMENT_INCOME_RATE).floor() as i64;
                self.wallet
                    .purchase_attachment(actor.id, post.user_id, post.id, price, author_income)
                    .await?;
            }
        }

        Ok(attachment_id)
    }
}

async fn post_attachment_price(post_id: i64, app: &AppContext) -> Result<i64, AppError> {
    let mut states = app.legacy_posts.post_states_by_ids(&[post_id]).await?;
    let state = states.remove(&post_id).unwrap_or_default();
    Ok(state.attachment_price.max(0))
}

fn parse_attachment_id(content: &str) -> Option<i64> {
    content
        .trim()
        .trim_end_matches('/')
        .rsplit('/')
        .next()?
        .parse()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::parse_attachment_id;

    #[test]
    fn parse_attachment_id_supports_legacy_content_url() {
        assert_eq!(parse_attachment_id("/v1/attachments/42"), Some(42));
        assert_eq!(
            parse_attachment_id("https://host/v1/attachments/7/"),
            Some(7)
        );
        assert_eq!(parse_attachment_id("invalid"), None);
    }
}
