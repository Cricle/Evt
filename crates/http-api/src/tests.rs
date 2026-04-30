use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use paopao_config::{
    AppSettings, DatabaseSettings, GrpcSettings, HttpSettings, JwtSettings, ServerSettings,
    Settings, SiteSettings, StorageSettings, WebSettings,
};
use tower::util::ServiceExt;

use crate::{HttpState, router};
use serde_json::Value;

fn openapi() -> Value {
    serde_json::from_str(include_str!("../../../docs/openapi.json")).unwrap()
}

fn test_settings() -> Settings {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let storage_dir = std::env::temp_dir().join(format!("paopao-http-api-tests-{unique}"));
    let web_dist_dir = std::env::temp_dir().join(format!("paopao-web-dist-tests-{unique}"));
    fs::create_dir_all(web_dist_dir.join("assets")).expect("create test web dist");
    fs::write(
        web_dist_dir.join("index.html"),
        "<!doctype html><html><body>paopao-web-shell</body></html>",
    )
    .expect("write index.html");
    fs::write(
        web_dist_dir.join("assets/app.js"),
        "console.log('paopao-web-asset');",
    )
    .expect("write asset");

    Settings {
        app: AppSettings {
            name: "paopao-rust".into(),
            env: "test".into(),
        },
        server: ServerSettings {
            http: HttpSettings {
                host: "127.0.0.1".into(),
                port: 18080,
            },
            grpc: GrpcSettings {
                host: "127.0.0.1".into(),
                port: 19090,
            },
        },
        database: DatabaseSettings {
            url: "mysql://paopao:paopao@127.0.0.1:3306/paopao".into(),
            max_connections: 1,
        },
        jwt: JwtSettings {
            secret: "test-secret".into(),
            issuer: "paopao-test".into(),
            expire_seconds: 3600,
        },
        storage: StorageSettings {
            local_dir: storage_dir.display().to_string(),
        },
        web: WebSettings {
            dist_dir: web_dist_dir.display().to_string(),
        },
        site: SiteSettings {
            allow_user_register: true,
            allow_phone_bind: false,
            use_friendship: true,
            enable_trends_bar: true,
            enable_wallet: false,
            allow_tweet_attachment: true,
            allow_tweet_attachment_price: false,
            allow_tweet_video: false,
            default_tweet_max_length: 280,
            tweet_web_ellipsis_size: 140,
            tweet_mobile_ellipsis_size: 70,
            default_tweet_visibility: "public".into(),
            default_msg_loop_interval: 15,
            copyright_top: "top".into(),
            copyright_left: "left".into(),
            copyright_left_link: "https://example.com/left".into(),
            copyright_right: "right".into(),
            copyright_right_link: "https://example.com/right".into(),
        },
    }
}

async fn test_app() -> axum::Router {
    let app = paopao_infra::AppContext::bootstrap_lazy(test_settings())
        .await
        .expect("build lazy app context");
    router(HttpState::new(app))
}

async fn response_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");
    serde_json::from_slice(&bytes).expect("parse response json")
}

#[tokio::test]
async fn root_path_serves_web_shell() {
    let app = test_app().await;
    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");
    let body = String::from_utf8(bytes.to_vec()).expect("decode response body");
    assert!(body.contains("paopao-web-shell"));
}

#[tokio::test]
async fn static_asset_path_serves_web_asset() {
    let app = test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/assets/app.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");
    let body = String::from_utf8(bytes.to_vec()).expect("decode response body");
    assert!(body.contains("paopao-web-asset"));
}

#[tokio::test]
async fn version_endpoint_keeps_success_envelope_shape() {
    let app = test_app().await;
    let response = app
        .oneshot(Request::builder().uri("/v1").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["code"], 0);
    assert_eq!(body["msg"], "success");
    assert_eq!(body["data"]["name"], "paopao-rust");
    assert_eq!(body["data"]["environment"], "test");
    assert!(body["data"]["version"].as_str().is_some());
}

#[tokio::test]
async fn site_profile_endpoint_keeps_existing_fields() {
    let app = test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/site/profile")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["code"], 0);
    assert_eq!(body["msg"], "success");
    assert_eq!(body["data"]["allow_user_register"], true);
    assert_eq!(body["data"]["use_friendship"], true);
    assert_eq!(body["data"]["default_tweet_max_length"], 280);
    assert_eq!(body["data"]["default_tweet_visibility"], "public");
    assert_eq!(
        body["data"]["copyright_right_link"],
        "https://example.com/right"
    );
}

#[tokio::test]
async fn legacy_user_info_route_exists_and_requires_auth() {
    let app = test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/user/info")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = response_json(response).await;
    assert_eq!(body["code"], 401001);
}

#[tokio::test]
async fn legacy_attachment_route_exists_and_requires_auth() {
    let app = test_app().await;
    let boundary = "x-boundary";
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/attachment")
                .header(
                    "content-type",
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(format!("--{boundary}--\r\n")))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = response_json(response).await;
    assert_eq!(body["code"], 401001);
}

#[tokio::test]
async fn legacy_profile_and_social_routes_exist_with_expected_guard_behavior() {
    let app = test_app().await;

    for path in [
        "/v1/user/profile",
        "/v1/user/posts",
        "/v1/user/follows",
        "/v1/user/followings",
        "/v1/tags",
    ] {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "path {path}");
    }

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/user/follow")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"user_id":1}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/suggest/users?k=")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    for path in ["/v1/user/contacts", "/v1/trends/index"] {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "path {path}");
    }

    for path in [
        "/v1/user/collections",
        "/v1/post/star?id=1",
        "/v1/post/collection?id=1",
    ] {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "path {path}");
    }

    for path in ["/v1/suggest/tags?k="] {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK, "path {path}");
    }

    for path in ["/v1/post/star", "/v1/post/collection"] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(path)
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"id":1}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "path {path}");
    }
}

#[tokio::test]
async fn protected_routes_reject_missing_bearer_token_with_legacy_error_shape() {
    let app = test_app().await;

    for path in ["/v1/feed", "/v1/users/me", "/v1/messages/unread-count"] {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "path {path}");

        let body = response_json(response).await;
        assert_eq!(body["code"], 401001, "path {path}");
        assert_eq!(body["msg"], "missing bearer token", "path {path}");
        assert!(body["data"].is_null(), "path {path}");
    }
}

#[tokio::test]
async fn protected_routes_reject_invalid_bearer_token_before_database_access() {
    let app = test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/feed")
                .header("authorization", "Bearer not-a-jwt")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = response_json(response).await;
    assert_eq!(body["code"], 401001);
    assert!(
        body["msg"]
            .as_str()
            .expect("error message")
            .starts_with("invalid token:")
    );
    assert!(body["data"].is_null());
}

#[tokio::test]
async fn legacy_web_compat_routes_exist_with_expected_auth_behavior() {
    let app = test_app().await;

    for path in [
        "/v1/user/messages",
        "/v1/user/msgcount/unread",
        "/v1/attachment?id=1",
        "/v1/attachment/precheck?id=1",
        "/v1/user/wallet/bills",
        "/v1/admin/site/status",
    ] {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "path {path}");
    }

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/captcha")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    for (path, body) in [
        ("/v1/user/whisper", r#"{"user_id":1,"content":"hi"}"#),
        ("/v1/friend/requesting", r#"{"user_id":1,"greetings":"hi"}"#),
        ("/v1/user/nickname", r#"{"nickname":"abc"}"#),
        ("/v1/post/lock", r#"{"id":1}"#),
        (
            "/v1/post/comment/reply",
            r#"{"comment_id":1,"content":"hi","at_user_id":0}"#,
        ),
        ("/v1/admin/settings/save", r#"{"items":[]}"#),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(path)
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "path {path}");
    }

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/post/comment/reply")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"id":1}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn captcha_post_keeps_legacy_invalid_code_shape() {
    let app = test_app().await;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/captcha")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    let captcha_id = body["data"]["id"].as_str().expect("captcha id");

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/captcha")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"phone":"13800138000","img_captcha":"wrong","img_captcha_id":"{captcha_id}"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = response_json(response).await;
    assert_eq!(body["code"], 20012);
    assert_eq!(body["msg"], "图形验证码验证失败");
    assert!(body["data"].is_null());
}

#[tokio::test]
async fn embedded_openapi_endpoint_matches_checked_in_spec() {
    let app = test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/docs/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["openapi"], "3.1.0");
    assert!(body["paths"]["/v1/messages"].is_object());
    assert!(body["paths"]["/v1/users/{username}/profile"].is_object());
}

#[test]
fn openapi_includes_social_web_paths() {
    let doc = openapi();
    let paths = doc["paths"].as_object().unwrap();

    for path in [
        "/v1/feed",
        "/v1/users/{username}/followers",
        "/v1/users/{username}/followings",
        "/v1/users/{username}/follow",
        "/v1/users/{username}/unfollow",
    ] {
        assert!(paths.contains_key(path), "missing path {path}");
    }
}

#[test]
fn openapi_marks_authenticated_social_paths() {
    let doc = openapi();
    let paths = doc["paths"].as_object().unwrap();

    for (path, method) in [
        ("/v1/feed", "get"),
        ("/v1/users/{username}/follow", "post"),
        ("/v1/users/{username}/unfollow", "post"),
    ] {
        let operation = &paths[path][method];
        assert!(
            operation["security"].is_array(),
            "missing bearer security for {method} {path}"
        );
    }
}

#[test]
fn openapi_includes_attachment_paths() {
    let doc = openapi();
    let paths = doc["paths"].as_object().unwrap();

    for path in ["/v1/attachments", "/v1/attachments/{attachment_id}"] {
        assert!(paths.contains_key(path), "missing path {path}");
    }
}

#[test]
fn openapi_describes_multipart_attachment_upload() {
    let doc = openapi();
    let operation = &doc["paths"]["/v1/attachments"]["post"];

    assert!(
        operation["security"].is_array(),
        "upload must require bearer auth"
    );
    assert_eq!(
        operation["requestBody"]["content"]["multipart/form-data"]["schema"]["type"],
        "object"
    );
}

#[test]
fn openapi_includes_post_moderation_paths() {
    let doc = openapi();
    let paths = doc["paths"].as_object().unwrap();

    for path in ["/v1/posts/{post_id}", "/v1/comments/{comment_id}"] {
        assert!(paths.contains_key(path), "missing path {path}");
    }
}

#[test]
fn openapi_marks_post_moderation_paths_authenticated() {
    let doc = openapi();

    for (path, method) in [
        ("/v1/posts/{post_id}", "patch"),
        ("/v1/posts/{post_id}", "delete"),
        ("/v1/comments/{comment_id}", "delete"),
    ] {
        let operation = &doc["paths"][path][method];
        assert!(
            operation["security"].is_array(),
            "missing bearer security for {method} {path}"
        );
    }
}

#[test]
fn openapi_includes_message_paths() {
    let doc = openapi();
    let paths = doc["paths"].as_object().unwrap();

    for path in [
        "/v1/messages",
        "/v1/messages/unread-count",
        "/v1/messages/{message_id}/read",
        "/v1/messages/read-all",
    ] {
        assert!(paths.contains_key(path), "missing path {path}");
    }
}

#[test]
fn openapi_marks_message_paths_authenticated() {
    let doc = openapi();

    for (path, method) in [
        ("/v1/messages", "get"),
        ("/v1/messages", "post"),
        ("/v1/messages/unread-count", "get"),
        ("/v1/messages/{message_id}/read", "post"),
        ("/v1/messages/read-all", "post"),
    ] {
        let operation = &doc["paths"][path][method];
        assert!(
            operation["security"].is_array(),
            "missing bearer security for {method} {path}"
        );
    }
}
