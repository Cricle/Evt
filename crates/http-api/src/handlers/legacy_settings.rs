use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
    time::{Duration, Instant},
};

use axum::{
    Json,
    body::Body,
    extract::Path,
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::Response,
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use rand::{Rng, distributions::Alphanumeric};
use serde::{Deserialize, Serialize};

use crate::{
    auth::authenticate_request,
    response::{ApiEnvelope, HttpApiError, legacy_error, success},
    state::HttpState,
};

static IMAGE_CAPTCHAS: LazyLock<Mutex<HashMap<String, (String, Instant)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static PHONE_CAPTCHAS: LazyLock<Mutex<HashMap<String, (String, Instant)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static ATTACHMENT_DOWNLOAD_TICKETS: LazyLock<Mutex<HashMap<String, (i64, Instant)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Deserialize)]
pub struct AttachmentQuery {
    id: i64,
}

#[derive(Debug, Deserialize)]
pub struct NicknameBody {
    nickname: String,
}

#[derive(Debug, Deserialize)]
pub struct AvatarBody {
    avatar: String,
}

#[derive(Debug, Deserialize)]
pub struct PasswordBody {
    password: String,
    old_password: String,
}

#[derive(Debug, Deserialize)]
pub struct PhoneBody {
    phone: String,
    captcha: String,
}

#[derive(Debug, Deserialize)]
pub struct ActivateBody {
    activate_code: String,
    captcha_id: String,
    #[serde(rename = "imgCaptcha")]
    img_captcha: String,
}

#[derive(Debug, Deserialize)]
pub struct SendCaptchaBody {
    phone: String,
    img_captcha: String,
    img_captcha_id: String,
}

#[derive(Debug, Serialize)]
pub struct AttachmentPrecheckResponse {
    paid: bool,
}

#[derive(Debug, Serialize)]
pub struct AttachmentDownloadResponse {
    signed_url: String,
}

#[derive(Debug, Serialize)]
pub struct CaptchaResponse {
    id: String,
    b64s: String,
}

pub async fn attachment_precheck(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Query(query): Query<AttachmentQuery>,
) -> Result<Json<ApiEnvelope<AttachmentPrecheckResponse>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let paid = state
        .app()
        .attachment_download_precheck(&actor, query.id)
        .await?;
    Ok(Json(success(AttachmentPrecheckResponse { paid })))
}

pub async fn attachment_get(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Query(query): Query<AttachmentQuery>,
) -> Result<Json<ApiEnvelope<AttachmentDownloadResponse>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let attachment_id = state
        .app()
        .resolve_attachment_id_from_content(&actor, query.id)
        .await?;
    let ticket = random_token(32);
    let mut tickets = ATTACHMENT_DOWNLOAD_TICKETS
        .lock()
        .expect("attachment ticket lock poisoned");
    let now = Instant::now();
    tickets.retain(|_, (_, expires_at)| now <= *expires_at);
    tickets.insert(
        ticket.clone(),
        (attachment_id, now + Duration::from_secs(60)),
    );
    Ok(Json(success(AttachmentDownloadResponse {
        signed_url: format!("/v1/attachment/download/{}", ticket),
    })))
}

pub async fn attachment_ticket_download(
    State(state): State<HttpState>,
    Path(ticket): Path<String>,
) -> Result<Response, HttpApiError> {
    let attachment_id = consume_attachment_ticket(&ticket)?;
    let attachment = state.app().download_attachment(attachment_id).await?;

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

pub async fn user_nickname(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<NicknameBody>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    state
        .app()
        .change_nickname(&actor, &payload.nickname)
        .await?;
    Ok(Json(success(serde_json::json!({}))))
}

pub async fn user_avatar(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<AvatarBody>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    state.app().update_avatar(&actor, &payload.avatar).await?;
    Ok(Json(success(serde_json::json!({}))))
}

pub async fn user_password(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<PasswordBody>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    state
        .app()
        .change_password(&actor, &payload.old_password, &payload.password)
        .await?;
    Ok(Json(success(serde_json::json!({}))))
}

pub async fn user_phone(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<PhoneBody>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    verify_phone_captcha(&payload.phone, &payload.captcha)?;
    state.app().bind_phone(&actor, &payload.phone).await?;
    Ok(Json(success(serde_json::json!({}))))
}

pub async fn user_activate(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<ActivateBody>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    verify_image_captcha(&payload.captcha_id, &payload.img_captcha)?;
    state
        .app()
        .activate_user(&actor, &payload.activate_code)
        .await?;
    Ok(Json(success(serde_json::json!({}))))
}

pub async fn captcha_get() -> Result<Json<ApiEnvelope<CaptchaResponse>>, HttpApiError> {
    let id = random_token(24);
    let code = random_numeric(6);
    IMAGE_CAPTCHAS
        .lock()
        .expect("captcha lock poisoned")
        .insert(
            id.clone(),
            (code.clone(), Instant::now() + Duration::from_secs(300)),
        );

    let svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' width='160' height='64'><rect width='100%' height='100%' fill='#daf0e4'/><text x='16' y='42' font-size='28' fill='#111' font-family='monospace'>{code}</text></svg>"
    );
    Ok(Json(success(CaptchaResponse {
        id,
        b64s: format!(
            "data:image/svg+xml;base64,{}",
            STANDARD.encode(svg.as_bytes())
        ),
    })))
}

pub async fn captcha_post(
    Json(payload): Json<SendCaptchaBody>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    verify_image_captcha(&payload.img_captcha_id, &payload.img_captcha)?;
    PHONE_CAPTCHAS
        .lock()
        .expect("captcha lock poisoned")
        .insert(
            payload.phone,
            (random_numeric(6), Instant::now() + Duration::from_secs(300)),
        );
    Ok(Json(success(serde_json::json!({}))))
}

fn verify_image_captcha(id: &str, input: &str) -> Result<(), HttpApiError> {
    let mut store = IMAGE_CAPTCHAS.lock().expect("captcha lock poisoned");
    let Some((expected, expires_at)) = store.remove(id) else {
        return Err(captcha_error());
    };
    if Instant::now() > expires_at || expected != input.trim() {
        return Err(captcha_error());
    }
    Ok(())
}

fn verify_phone_captcha(phone: &str, input: &str) -> Result<(), HttpApiError> {
    let mut store = PHONE_CAPTCHAS.lock().expect("captcha lock poisoned");
    let Some((expected, expires_at)) = store.remove(phone) else {
        return Err(phone_captcha_error());
    };
    if Instant::now() > expires_at || expected != input.trim() {
        return Err(phone_captcha_error());
    }
    Ok(())
}

fn captcha_error() -> HttpApiError {
    legacy_error(StatusCode::BAD_REQUEST, 20012, "图形验证码验证失败")
}

fn phone_captcha_error() -> HttpApiError {
    legacy_error(StatusCode::BAD_REQUEST, 20018, "手机验证码不正确")
}

fn consume_attachment_ticket(ticket: &str) -> Result<i64, HttpApiError> {
    let mut store = ATTACHMENT_DOWNLOAD_TICKETS
        .lock()
        .expect("attachment ticket lock poisoned");
    let Some((attachment_id, expires_at)) = store.remove(ticket) else {
        return Err(
            evt_domain::AppError::NotFound("attachment download ticket not found".into()).into(),
        );
    };
    if Instant::now() > expires_at {
        return Err(evt_domain::AppError::Unauthorized(
            "attachment download ticket expired".into(),
        )
        .into());
    }
    Ok(attachment_id)
}

fn random_token(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

fn random_numeric(len: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| char::from(b'0' + rng.gen_range(0..10) as u8))
        .collect()
}
