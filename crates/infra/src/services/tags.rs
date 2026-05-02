use evt_domain::{AppError, TagSummary, UserIdentity};

use crate::AppContext;

impl AppContext {
    async fn ensure_tag_belongs_to_space(
        &self,
        space_id: i64,
        tag_id: i64,
    ) -> Result<(), AppError> {
        let tag = self
            .tags
            .find_by_id(tag_id)
            .await?
            .ok_or_else(|| AppError::NotFound("topic not found".into()))?;
        if tag.space_id != space_id {
            return Err(AppError::NotFound("topic not found".into()));
        }
        Ok(())
    }

    pub async fn suggest_tags(&self, keyword: &str, limit: u64) -> Result<Vec<String>, AppError> {
        self.suggest_tags_in_space(None, None, keyword, limit).await
    }

    pub async fn suggest_tags_in_space(
        &self,
        viewer: Option<&UserIdentity>,
        space_slug: Option<&str>,
        keyword: &str,
        limit: u64,
    ) -> Result<Vec<String>, AppError> {
        let keyword = keyword.trim();
        if keyword.is_empty() {
            return Ok(Vec::new());
        }

        let space_id = self.resolve_space(viewer, space_slug).await?.id;
        self.tags.suggest(space_id, keyword, limit).await
    }

    pub async fn list_tags(
        &self,
        tag_type: &str,
        num: u64,
        extra_num: u64,
        actor: Option<&UserIdentity>,
    ) -> Result<(Vec<TagSummary>, Vec<TagSummary>), AppError> {
        self.list_tags_in_space(None, tag_type, num, extra_num, actor)
            .await
    }

    pub async fn list_tags_in_space(
        &self,
        space_slug: Option<&str>,
        tag_type: &str,
        num: u64,
        extra_num: u64,
        actor: Option<&UserIdentity>,
    ) -> Result<(Vec<TagSummary>, Vec<TagSummary>), AppError> {
        let space_id = self.resolve_space(actor, space_slug).await?.id;
        match tag_type {
            "new" => Ok((self.tags.list_new(space_id, num).await?, Vec::new())),
            "follow" => Ok((
                match actor {
                    Some(actor) => self.tags.list_followed(space_id, actor.id, num).await?,
                    None => Vec::new(),
                },
                Vec::new(),
            )),
            "pin" => Ok((
                match actor {
                    Some(actor) => self.tags.list_pinned(space_id, actor.id, num).await?,
                    None => Vec::new(),
                },
                Vec::new(),
            )),
            "hot_extral" => {
                self.tags
                    .list_hot_with_followed(space_id, actor.map(|item| item.id), num, extra_num)
                    .await
            }
            _ => Ok((self.tags.list_hot(space_id, num).await?, Vec::new())),
        }
    }

    pub async fn follow_tag(&self, actor: &UserIdentity, tag_id: i64) -> Result<(), AppError> {
        self.follow_tag_in_space(actor, None, tag_id).await
    }

    pub async fn follow_tag_in_space(
        &self,
        actor: &UserIdentity,
        space_slug: Option<&str>,
        tag_id: i64,
    ) -> Result<(), AppError> {
        let space_id = self.resolve_space(Some(actor), space_slug).await?.id;
        self.ensure_tag_belongs_to_space(space_id, tag_id).await?;
        self.tags.follow(space_id, actor.id, tag_id).await
    }

    pub async fn unfollow_tag(&self, actor: &UserIdentity, tag_id: i64) -> Result<(), AppError> {
        self.unfollow_tag_in_space(actor, None, tag_id).await
    }

    pub async fn unfollow_tag_in_space(
        &self,
        actor: &UserIdentity,
        space_slug: Option<&str>,
        tag_id: i64,
    ) -> Result<(), AppError> {
        let space_id = self.resolve_space(Some(actor), space_slug).await?.id;
        self.ensure_tag_belongs_to_space(space_id, tag_id).await?;
        self.tags.unfollow(space_id, actor.id, tag_id).await
    }

    pub async fn toggle_tag_top(
        &self,
        actor: &UserIdentity,
        tag_id: i64,
    ) -> Result<bool, AppError> {
        self.toggle_tag_top_in_space(actor, None, tag_id).await
    }

    pub async fn toggle_tag_top_in_space(
        &self,
        actor: &UserIdentity,
        space_slug: Option<&str>,
        tag_id: i64,
    ) -> Result<bool, AppError> {
        let space_id = self.resolve_space(Some(actor), space_slug).await?.id;
        self.ensure_tag_belongs_to_space(space_id, tag_id).await?;
        self.tags.toggle_top(space_id, actor.id, tag_id).await
    }

    pub async fn toggle_tag_pin(
        &self,
        actor: &UserIdentity,
        tag_id: i64,
    ) -> Result<bool, AppError> {
        self.toggle_tag_pin_in_space(actor, None, tag_id).await
    }

    pub async fn toggle_tag_pin_in_space(
        &self,
        actor: &UserIdentity,
        space_slug: Option<&str>,
        tag_id: i64,
    ) -> Result<bool, AppError> {
        let space_id = self.resolve_space(Some(actor), space_slug).await?.id;
        self.ensure_tag_belongs_to_space(space_id, tag_id).await?;
        self.tags.toggle_pin(space_id, actor.id, tag_id).await
    }
}

#[cfg(test)]
mod tests {
    use evt_domain::AppError;

    #[test]
    fn cross_space_topic_guard_uses_not_found_shape() {
        let error = AppError::NotFound("topic not found".into());
        assert_eq!(error.code(), 404_001);
        assert_eq!(error.to_string(), "topic not found");
    }
}
