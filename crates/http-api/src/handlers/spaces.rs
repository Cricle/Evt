use axum::{
    Json,
    extract::{Query, State},
    http::HeaderMap,
};
use evt_domain::{SpaceMemberSummary, SpaceRole, SpaceSummary, SpaceVisibility};
use serde::Deserialize;

use crate::{
    auth::{authenticate_optional_request, authenticate_request},
    response::{ApiEnvelope, HttpApiError, success},
    state::HttpState,
};

#[derive(Debug, Deserialize)]
pub struct ListSpacesQuery {
    pub limit: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSpaceRequest {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub visibility: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddSpaceMemberRequest {
    pub space_id: i64,
    pub username: String,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListSpaceMembersQuery {
    pub space_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSpaceMemberRequest {
    pub space_id: i64,
    pub user_id: i64,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RemoveSpaceMemberRequest {
    pub space_id: i64,
    pub user_id: i64,
}

pub async fn list_spaces(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Query(query): Query<ListSpacesQuery>,
) -> Result<Json<ApiEnvelope<Vec<SpaceSummary>>>, HttpApiError> {
    let actor = authenticate_optional_request(state.app(), &headers).await?;
    let spaces = state
        .app()
        .list_spaces(actor.as_ref(), query.limit.unwrap_or(50).clamp(1, 100))
        .await?;
    Ok(Json(success(spaces)))
}

pub async fn create_space(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<CreateSpaceRequest>,
) -> Result<Json<ApiEnvelope<SpaceSummary>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let visibility = parse_space_visibility(payload.visibility.as_deref());
    let space = state
        .app()
        .create_space(
            &actor,
            &payload.slug,
            &payload.name,
            payload.description.as_deref().unwrap_or_default(),
            visibility,
        )
        .await?;
    Ok(Json(success(space)))
}

pub async fn add_space_member(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<AddSpaceMemberRequest>,
) -> Result<Json<ApiEnvelope<SpaceMemberSummary>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let role = parse_space_role(payload.role.as_deref());
    let member = state
        .app()
        .add_space_member_by_username(&actor, payload.space_id, &payload.username, role)
        .await?;
    Ok(Json(success(member)))
}

pub async fn list_space_members(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Query(query): Query<ListSpaceMembersQuery>,
) -> Result<Json<ApiEnvelope<Vec<SpaceMemberSummary>>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let members = state
        .app()
        .list_space_members(&actor, query.space_id)
        .await?;
    Ok(Json(success(members)))
}

pub async fn update_space_member(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateSpaceMemberRequest>,
) -> Result<Json<ApiEnvelope<SpaceMemberSummary>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let role = parse_space_role(payload.role.as_deref());
    let member = state
        .app()
        .update_space_member_role(&actor, payload.space_id, payload.user_id, role)
        .await?;
    Ok(Json(success(member)))
}

pub async fn remove_space_member(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<RemoveSpaceMemberRequest>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    state
        .app()
        .remove_space_member(&actor, payload.space_id, payload.user_id)
        .await?;
    Ok(Json(success(serde_json::json!({}))))
}

fn parse_space_visibility(value: Option<&str>) -> SpaceVisibility {
    match value {
        Some("private") => SpaceVisibility::Private,
        _ => SpaceVisibility::Public,
    }
}

fn parse_space_role(value: Option<&str>) -> SpaceRole {
    match value {
        Some("admin") => SpaceRole::Admin,
        _ => SpaceRole::Member,
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_space_role, parse_space_visibility};
    use evt_domain::{SpaceRole, SpaceVisibility};

    #[test]
    fn parse_space_visibility_keeps_legacy_default_public() {
        assert_eq!(parse_space_visibility(None), SpaceVisibility::Public);
        assert_eq!(
            parse_space_visibility(Some("unexpected")),
            SpaceVisibility::Public
        );
        assert_eq!(
            parse_space_visibility(Some("private")),
            SpaceVisibility::Private
        );
    }

    #[test]
    fn parse_space_role_keeps_legacy_default_member() {
        assert_eq!(parse_space_role(None), SpaceRole::Member);
        assert_eq!(parse_space_role(Some("member")), SpaceRole::Member);
        assert_eq!(parse_space_role(Some("admin")), SpaceRole::Admin);
    }
}
