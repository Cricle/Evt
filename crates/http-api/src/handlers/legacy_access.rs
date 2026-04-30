use std::collections::HashMap;

use paopao_domain::{LegacyPostState, PostSummary, UserIdentity};
use paopao_infra::AppContext;

use crate::response::{HttpApiError, legacy_error};

pub(crate) const LEGACY_VISIBILITY_PUBLIC: i32 = 0;
pub(crate) const LEGACY_VISIBILITY_PRIVATE: i32 = 1;
pub(crate) const LEGACY_VISIBILITY_FRIEND: i32 = 2;
pub(crate) const LEGACY_VISIBILITY_FOLLOWING: i32 = 3;

pub(crate) fn legacy_no_permission() -> HttpApiError {
    legacy_error(http::StatusCode::BAD_REQUEST, 20007, "无权限执行该请求")
}

pub(crate) fn legacy_admin_only() -> HttpApiError {
    legacy_error(http::StatusCode::BAD_REQUEST, 20022, "无管理权限")
}

pub(crate) fn legacy_visibility(state: Option<&LegacyPostState>) -> i32 {
    state
        .map(|item| item.visibility)
        .unwrap_or(LEGACY_VISIBILITY_PUBLIC)
}

pub(crate) fn can_view_post(
    viewer: Option<&UserIdentity>,
    author_id: i64,
    visibility: i32,
    is_following: bool,
    is_friend: bool,
) -> bool {
    match viewer {
        Some(viewer) if viewer.id == author_id => true,
        Some(_) => match visibility {
            LEGACY_VISIBILITY_PUBLIC => true,
            LEGACY_VISIBILITY_PRIVATE => false,
            LEGACY_VISIBILITY_FRIEND => is_friend,
            LEGACY_VISIBILITY_FOLLOWING => is_following,
            _ => false,
        },
        None => visibility == LEGACY_VISIBILITY_PUBLIC,
    }
}

pub(crate) async fn relation_status(
    app: &AppContext,
    viewer: Option<&UserIdentity>,
    author_id: i64,
) -> Result<(bool, bool), HttpApiError> {
    let Some(viewer) = viewer else {
        return Ok((false, false));
    };
    if viewer.id == author_id {
        return Ok((false, false));
    }

    Ok((
        app.is_following(viewer.id, author_id).await?,
        app.is_friend(viewer.id, author_id).await?,
    ))
}

pub(crate) async fn batch_relation_maps(
    app: &AppContext,
    viewer: Option<&UserIdentity>,
    author_ids: &[i64],
) -> Result<(HashMap<i64, bool>, HashMap<i64, bool>), HttpApiError> {
    let Some(viewer) = viewer else {
        return Ok((HashMap::new(), HashMap::new()));
    };
    if author_ids.is_empty() {
        return Ok((HashMap::new(), HashMap::new()));
    }

    let following = app.batch_following_status(viewer.id, author_ids).await?;
    let friends = app.batch_friend_status(viewer.id, author_ids).await?;
    Ok((following, friends))
}

pub(crate) async fn ensure_can_view_post(
    app: &AppContext,
    viewer: Option<&UserIdentity>,
    post: &PostSummary,
) -> Result<(), HttpApiError> {
    let states = app.legacy_post_states_by_ids(&[post.id]).await?;
    let visibility = legacy_visibility(states.get(&post.id));
    let (is_following, is_friend) = relation_status(app, viewer, post.user_id).await?;
    if can_view_post(viewer, post.user_id, visibility, is_following, is_friend) {
        return Ok(());
    }

    Err(legacy_no_permission())
}

#[cfg(test)]
mod tests {
    use super::{
        LEGACY_VISIBILITY_FOLLOWING, LEGACY_VISIBILITY_FRIEND, LEGACY_VISIBILITY_PRIVATE,
        LEGACY_VISIBILITY_PUBLIC, can_view_post, legacy_no_permission, legacy_visibility,
    };
    use paopao_domain::{LegacyPostState, UserIdentity};

    #[test]
    fn visibility_defaults_to_public_without_state() {
        assert_eq!(legacy_visibility(None), LEGACY_VISIBILITY_PUBLIC);

        let state = LegacyPostState {
            visibility: LEGACY_VISIBILITY_FRIEND,
            ..Default::default()
        };
        assert_eq!(legacy_visibility(Some(&state)), LEGACY_VISIBILITY_FRIEND);
    }

    #[test]
    fn access_rules_match_legacy_visibility_contract() {
        let viewer = UserIdentity {
            id: 10,
            username: "alice".into(),
        };

        assert!(can_view_post(
            None,
            1,
            LEGACY_VISIBILITY_PUBLIC,
            false,
            false
        ));
        assert!(!can_view_post(
            None,
            1,
            LEGACY_VISIBILITY_PRIVATE,
            false,
            false
        ));
        assert!(!can_view_post(
            Some(&viewer),
            1,
            LEGACY_VISIBILITY_PRIVATE,
            false,
            false
        ));
        assert!(can_view_post(
            Some(&viewer),
            1,
            LEGACY_VISIBILITY_FRIEND,
            false,
            true
        ));
        assert!(can_view_post(
            Some(&viewer),
            1,
            LEGACY_VISIBILITY_FOLLOWING,
            true,
            false
        ));
        assert!(can_view_post(
            Some(&viewer),
            10,
            LEGACY_VISIBILITY_PRIVATE,
            false,
            false
        ));
    }

    #[test]
    fn forbidden_response_keeps_legacy_code() {
        match legacy_no_permission() {
            super::HttpApiError::Legacy { status, code, .. } => {
                assert_eq!(status, http::StatusCode::BAD_REQUEST);
                assert_eq!(code, 20007);
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }
}
