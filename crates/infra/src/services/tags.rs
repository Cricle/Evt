use evt_domain::{AppError, TagSummary, UserIdentity};

use crate::AppContext;

impl AppContext {
    pub async fn suggest_tags(&self, keyword: &str, limit: u64) -> Result<Vec<String>, AppError> {
        let keyword = keyword.trim();
        if keyword.is_empty() {
            return Ok(Vec::new());
        }

        self.tags.suggest(keyword, limit).await
    }

    pub async fn list_tags(
        &self,
        tag_type: &str,
        num: u64,
        extra_num: u64,
        actor: Option<&UserIdentity>,
    ) -> Result<(Vec<TagSummary>, Vec<TagSummary>), AppError> {
        match tag_type {
            "new" => Ok((self.tags.list_new(num).await?, Vec::new())),
            "follow" => Ok((
                match actor {
                    Some(actor) => self.tags.list_followed(actor.id, num).await?,
                    None => Vec::new(),
                },
                Vec::new(),
            )),
            "pin" => Ok((
                match actor {
                    Some(actor) => self.tags.list_pinned(actor.id, num).await?,
                    None => Vec::new(),
                },
                Vec::new(),
            )),
            "hot_extral" => {
                self.tags
                    .list_hot_with_followed(actor.map(|item| item.id), num, extra_num)
                    .await
            }
            _ => Ok((self.tags.list_hot(num).await?, Vec::new())),
        }
    }

    pub async fn follow_tag(&self, actor: &UserIdentity, tag_id: i64) -> Result<(), AppError> {
        self.tags.follow(actor.id, tag_id).await
    }

    pub async fn unfollow_tag(&self, actor: &UserIdentity, tag_id: i64) -> Result<(), AppError> {
        self.tags.unfollow(actor.id, tag_id).await
    }

    pub async fn toggle_tag_top(
        &self,
        actor: &UserIdentity,
        tag_id: i64,
    ) -> Result<bool, AppError> {
        self.tags.toggle_top(actor.id, tag_id).await
    }

    pub async fn toggle_tag_pin(
        &self,
        actor: &UserIdentity,
        tag_id: i64,
    ) -> Result<bool, AppError> {
        self.tags.toggle_pin(actor.id, tag_id).await
    }
}
