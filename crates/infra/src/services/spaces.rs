use evt_domain::{
    AppError, LEGACY_DEFAULT_SPACE_SLUG, PUBLIC_SPACE_SLUG, SpaceMemberSummary, SpaceRole,
    SpaceSummary, SpaceVisibility, UserIdentity,
};

use crate::AppContext;

impl AppContext {
    fn runtime_default_space_slug(&self) -> String {
        let slug = self.site_profile_snapshot().default_space_slug;
        normalize_space_slug(Some(slug.as_str()))
    }

    pub(crate) fn normalized_default_space_slug(&self) -> String {
        self.runtime_default_space_slug()
    }

    pub async fn default_space(&self) -> Result<SpaceSummary, AppError> {
        for candidate in default_space_slug_candidates(&self.runtime_default_space_slug()) {
            if let Some(space) = self.spaces.find_by_slug(&candidate).await? {
                return Ok(space);
            }
        }

        if let Some(space) = self.spaces.list_visible(None, 1).await?.into_iter().next() {
            return Ok(space);
        }

        Err(AppError::NotFound("default space not found".into()))
    }

    pub async fn default_space_id(&self) -> Result<i64, AppError> {
        Ok(self.default_space().await?.id)
    }

    pub async fn create_space(
        &self,
        actor: &UserIdentity,
        slug: &str,
        name: &str,
        description: &str,
        visibility: SpaceVisibility,
    ) -> Result<SpaceSummary, AppError> {
        let slug = slug.trim().to_lowercase();
        let name = name.trim();
        if slug.len() < 2 || name.len() < 2 {
            return Err(AppError::Validation(
                "space slug or name is too short".into(),
            ));
        }
        self.spaces
            .create(&slug, name, description.trim(), actor.id, visibility)
            .await
    }

    pub async fn list_spaces(
        &self,
        actor: Option<&UserIdentity>,
        limit: u64,
    ) -> Result<Vec<SpaceSummary>, AppError> {
        self.spaces
            .list_visible(actor.map(|item| item.id), limit)
            .await
    }

    pub async fn resolve_space(
        &self,
        viewer: Option<&UserIdentity>,
        slug: Option<&str>,
    ) -> Result<SpaceSummary, AppError> {
        let requested = slug.unwrap_or_default().trim();
        for candidate in requested_space_slug_candidates(slug, &self.runtime_default_space_slug()) {
            if let Some(space) = self.spaces.find_by_slug(&candidate).await? {
                self.ensure_can_access_space(viewer, &space).await?;
                return Ok(space);
            }
        }

        if requested.is_empty() {
            if let Some(space) = self
                .spaces
                .list_visible(viewer.map(|item| item.id), 1)
                .await?
                .into_iter()
                .next()
            {
                return Ok(space);
            }
        }

        Err(AppError::NotFound("space not found".into()))
    }

    pub async fn ensure_can_access_space(
        &self,
        viewer: Option<&UserIdentity>,
        space: &SpaceSummary,
    ) -> Result<(), AppError> {
        if matches!(space.visibility, SpaceVisibility::Public) {
            return Ok(());
        }

        let Some(viewer) = viewer else {
            return Err(AppError::Unauthorized("space membership required".into()));
        };

        if viewer.id == space.owner_user_id {
            return Ok(());
        }

        if self
            .spaces
            .find_member(space.id, viewer.id)
            .await?
            .is_some()
        {
            return Ok(());
        }

        Err(AppError::Unauthorized("space membership required".into()))
    }

    pub async fn ensure_can_access_space_id(
        &self,
        viewer: Option<&UserIdentity>,
        space_id: i64,
    ) -> Result<(), AppError> {
        let space = self
            .spaces
            .find_by_id(space_id)
            .await?
            .ok_or_else(|| AppError::NotFound("space not found".into()))?;
        self.ensure_can_access_space(viewer, &space).await
    }

    pub async fn add_space_member_by_username(
        &self,
        actor: &UserIdentity,
        space_id: i64,
        username: &str,
        role: SpaceRole,
    ) -> Result<SpaceMemberSummary, AppError> {
        let actor_member = self
            .spaces
            .find_member(space_id, actor.id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("space membership required".into()))?;

        if !actor_member.role.can_manage_members() {
            return Err(AppError::Unauthorized(
                "space admin permission required".into(),
            ));
        }

        let target = self
            .users
            .find_summary_by_username(username)
            .await?
            .ok_or_else(|| AppError::NotFound("target user not found".into()))?;

        let next_role = match role {
            SpaceRole::Owner => SpaceRole::Admin,
            other => other,
        };

        self.spaces
            .add_member(space_id, target.id, next_role, actor.id)
            .await?;

        self.spaces
            .find_member(space_id, target.id)
            .await?
            .ok_or_else(|| AppError::Internal("space member cannot be loaded".into()))
    }

    pub async fn list_space_members(
        &self,
        actor: &UserIdentity,
        space_id: i64,
    ) -> Result<Vec<SpaceMemberSummary>, AppError> {
        let actor_member = self.ensure_space_member(actor, space_id).await?;
        if !actor_member.role.can_manage_members() {
            return Err(AppError::Unauthorized(
                "space admin permission required".into(),
            ));
        }
        self.spaces.list_members(space_id).await
    }

    pub async fn update_space_member_role(
        &self,
        actor: &UserIdentity,
        space_id: i64,
        user_id: i64,
        role: SpaceRole,
    ) -> Result<SpaceMemberSummary, AppError> {
        let actor_member = self.ensure_space_member(actor, space_id).await?;
        if !actor_member.role.can_manage_members() {
            return Err(AppError::Unauthorized(
                "space admin permission required".into(),
            ));
        }
        let space = self
            .spaces
            .find_by_id(space_id)
            .await?
            .ok_or_else(|| AppError::NotFound("space not found".into()))?;
        if user_id == space.owner_user_id {
            return Err(AppError::Validation("owner role cannot be changed".into()));
        }

        let next_role = match role {
            SpaceRole::Owner => SpaceRole::Admin,
            other => other,
        };
        self.spaces
            .add_member(space_id, user_id, next_role, actor.id)
            .await?;
        self.spaces
            .find_member(space_id, user_id)
            .await?
            .ok_or_else(|| AppError::Internal("space member cannot be loaded".into()))
    }

    pub async fn remove_space_member(
        &self,
        actor: &UserIdentity,
        space_id: i64,
        user_id: i64,
    ) -> Result<(), AppError> {
        let actor_member = self.ensure_space_member(actor, space_id).await?;
        if !actor_member.role.can_manage_members() {
            return Err(AppError::Unauthorized(
                "space admin permission required".into(),
            ));
        }
        let space = self
            .spaces
            .find_by_id(space_id)
            .await?
            .ok_or_else(|| AppError::NotFound("space not found".into()))?;
        if user_id == space.owner_user_id {
            return Err(AppError::Validation("owner cannot be removed".into()));
        }
        self.spaces.remove_member(space_id, user_id).await
    }

    pub async fn ensure_space_member(
        &self,
        actor: &UserIdentity,
        space_id: i64,
    ) -> Result<SpaceMemberSummary, AppError> {
        self.ensure_can_access_space_id(Some(actor), space_id)
            .await?;
        self.spaces
            .find_member(space_id, actor.id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("space membership required".into()))
    }
}

fn normalize_space_slug(slug: Option<&str>) -> String {
    let normalized = slug.unwrap_or_default().trim().to_lowercase();
    if normalized.is_empty() || normalized == LEGACY_DEFAULT_SPACE_SLUG {
        PUBLIC_SPACE_SLUG.to_string()
    } else {
        normalized
    }
}

fn default_space_slug_candidates(runtime_default_slug: &str) -> Vec<String> {
    let mut candidates = vec![normalize_space_slug(Some(runtime_default_slug))];
    if !candidates.iter().any(|item| item == PUBLIC_SPACE_SLUG) {
        candidates.push(PUBLIC_SPACE_SLUG.to_string());
    }
    if !candidates
        .iter()
        .any(|item| item == LEGACY_DEFAULT_SPACE_SLUG)
    {
        candidates.push(LEGACY_DEFAULT_SPACE_SLUG.to_string());
    }
    candidates
}

fn requested_space_slug_candidates(
    requested_slug: Option<&str>,
    runtime_default_slug: &str,
) -> Vec<String> {
    let requested = requested_slug.unwrap_or_default().trim();
    if requested.is_empty() {
        return default_space_slug_candidates(runtime_default_slug);
    }

    let normalized = requested.to_lowercase();
    let mut candidates = vec![normalized.clone()];

    if normalized == PUBLIC_SPACE_SLUG {
        candidates.push(LEGACY_DEFAULT_SPACE_SLUG.to_string());
    } else if normalized == LEGACY_DEFAULT_SPACE_SLUG {
        candidates.push(PUBLIC_SPACE_SLUG.to_string());
    }

    candidates
}

#[cfg(test)]
mod tests {
    use super::{
        default_space_slug_candidates, normalize_space_slug, requested_space_slug_candidates,
    };

    #[test]
    fn normalize_space_slug_maps_empty_and_legacy_slug_to_public() {
        assert_eq!(normalize_space_slug(None), "public");
        assert_eq!(normalize_space_slug(Some("")), "public");
        assert_eq!(normalize_space_slug(Some("square")), "public");
        assert_eq!(normalize_space_slug(Some("  SQUARE ")), "public");
        assert_eq!(normalize_space_slug(Some("Team-Alpha")), "team-alpha");
    }

    #[test]
    fn default_space_candidates_keep_public_and_legacy_alias() {
        assert_eq!(
            default_space_slug_candidates("public"),
            vec!["public".to_string(), "square".to_string()]
        );
        assert_eq!(
            default_space_slug_candidates("square"),
            vec!["public".to_string(), "square".to_string()]
        );
        assert_eq!(
            default_space_slug_candidates("team-alpha"),
            vec![
                "team-alpha".to_string(),
                "public".to_string(),
                "square".to_string(),
            ]
        );
    }

    #[test]
    fn requested_space_candidates_support_bidirectional_public_alias() {
        assert_eq!(
            requested_space_slug_candidates(Some("public"), "public"),
            vec!["public".to_string(), "square".to_string()]
        );
        assert_eq!(
            requested_space_slug_candidates(Some("square"), "public"),
            vec!["square".to_string(), "public".to_string()]
        );
        assert_eq!(
            requested_space_slug_candidates(Some("team-alpha"), "public"),
            vec!["team-alpha".to_string()]
        );
        assert_eq!(
            requested_space_slug_candidates(None, "team-alpha"),
            vec![
                "team-alpha".to_string(),
                "public".to_string(),
                "square".to_string(),
            ]
        );
    }

    #[test]
    fn normalized_default_space_slug_always_maps_empty_and_legacy_alias_to_public() {
        assert_eq!(normalize_space_slug(Some("")), "public");
        assert_eq!(normalize_space_slug(Some("square")), "public");
        assert_eq!(normalize_space_slug(Some("  square  ")), "public");
    }
}
