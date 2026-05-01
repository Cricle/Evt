use axum::{
    Json,
    body::Body,
    extract::{Multipart, Path, State},
    http::{HeaderMap, HeaderValue, header},
    response::Response,
};
use evt_domain::{AppError, AttachmentSummary};

use crate::{
    auth::authenticate_request,
    response::{ApiEnvelope, HttpApiError, success},
    state::HttpState,
};

pub async fn upload_attachment(
    State(state): State<HttpState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<ApiEnvelope<AttachmentSummary>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let mut upload_type = "attachment".to_string();

    while let Some(field) = multipart.next_field().await.map_err(|err| {
        HttpApiError::from(AppError::Validation(format!(
            "invalid multipart body: {err}"
        )))
    })? {
        if field.name() == Some("type") {
            upload_type = field.text().await.unwrap_or_else(|_| "attachment".into());
            continue;
        }

        if field.name() != Some("file") {
            continue;
        }

        let file_name = field.file_name().unwrap_or("file").to_string();
        let content_type = field.content_type().map(ToOwned::to_owned);
        let bytes = field.bytes().await.map_err(|err| {
            HttpApiError::from(AppError::Validation(format!(
                "read multipart field failed: {err}"
            )))
        })?;

        let attachment = state
            .app()
            .upload_attachment_by_kind(
                &actor,
                upload_type.as_str(),
                &file_name,
                content_type.as_deref(),
                &bytes,
            )
            .await?;
        return Ok(Json(success(attachment)));
    }

    Err(AppError::Validation("multipart field `file` is required".into()).into())
}

pub async fn download_attachment(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Path(attachment_id): Path<i64>,
) -> Result<Response, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let attachment = state
        .app()
        .download_attachment_with_access(&actor, attachment_id)
        .await?;
    let mut response = Response::new(Body::from(attachment.bytes));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&attachment.content_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    let disposition = format!("attachment; filename=\"{}\"", attachment.file_name);
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&disposition)
            .unwrap_or_else(|_| HeaderValue::from_static("attachment")),
    );
    Ok(response)
}
