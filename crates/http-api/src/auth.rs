use axum::http::HeaderMap;
use evt_domain::{AppError, UserIdentity};
use evt_infra::AppContext;

use crate::response::HttpApiError;

pub async fn authenticate_request(
    app: &AppContext,
    headers: &HeaderMap,
) -> Result<UserIdentity, HttpApiError> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;

    let identity = app.authenticate_token(token).await?;
    app.mark_online(identity.id);
    Ok(identity)
}

pub async fn authenticate_optional_request(
    app: &AppContext,
    headers: &HeaderMap,
) -> Result<Option<UserIdentity>, HttpApiError> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));

    match token {
        Some(token) if !token.is_empty() => {
            let identity = app.authenticate_token(token).await?;
            app.mark_online(identity.id);
            Ok(Some(identity))
        }
        _ => Ok(None),
    }
}
