use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use paopao_domain::AppError;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiEnvelope<T> {
    code: i32,
    msg: String,
    data: Option<T>,
}

#[derive(Debug)]
pub enum HttpApiError {
    App(AppError),
    Legacy {
        status: StatusCode,
        code: i32,
        msg: String,
    },
}

impl From<AppError> for HttpApiError {
    fn from(value: AppError) -> Self {
        Self::App(value)
    }
}

impl IntoResponse for HttpApiError {
    fn into_response(self) -> Response {
        let (status, code, msg) = match self {
            Self::App(err) => (err.status_code(), err.code(), err.to_string()),
            Self::Legacy { status, code, msg } => (status, code, msg),
        };
        let body = Json(ApiEnvelope::<()> {
            code,
            msg,
            data: None,
        });
        (status, body).into_response()
    }
}

pub fn legacy_error(status: StatusCode, code: i32, msg: impl Into<String>) -> HttpApiError {
    HttpApiError::Legacy {
        status,
        code,
        msg: msg.into(),
    }
}

pub fn success<T>(data: T) -> ApiEnvelope<T> {
    ApiEnvelope {
        code: 0,
        msg: "success".into(),
        data: Some(data),
    }
}
