use std::{
    fs::File,
    net::TcpListener as StdTcpListener,
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{Mutex, MutexGuard, OnceLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use evt_config::{
    AppSettings, DatabaseSettings, GrpcSettings, HttpSettings, JwtSettings, ServerSettings,
    Settings, SiteSettings, StorageSettings, TelemetrySettings, WebSettings,
};
use evt_grpc_api::proto::message_service_client::MessageServiceClient;
use evt_grpc_api::proto::{
    ListLegacyMessagesRequest, MarkReadRequest, SendLegacyWhisperRequest, UnreadCountRequest,
};
use reqwest::StatusCode;
use reqwest::multipart;
use serde_json::Value;
use tokio::{fs, time::sleep};

fn unique_suffix() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_millis()
        .to_string()
}

fn reserve_port() -> u16 {
    let listener = StdTcpListener::bind("127.0.0.1:0").expect("reserve local port");
    let port = listener.local_addr().expect("listener addr").port();
    drop(listener);
    port
}

fn mysql_env(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

fn mysql_database_url(database: &str) -> String {
    format!(
        "mysql://{}:{}@{}:{}/{}",
        mysql_env("MYSQL_USER", "evt"),
        mysql_env("MYSQL_PASSWORD", "evt"),
        mysql_env("MYSQL_HOST", "127.0.0.1"),
        mysql_env("MYSQL_PORT", "3306"),
        database
    )
}

fn mysql_args(database: Option<&str>, sql: &str) -> Vec<String> {
    let mut args = vec![
        format!("--host={}", mysql_env("MYSQL_HOST", "127.0.0.1")),
        format!("--port={}", mysql_env("MYSQL_PORT", "3306")),
        format!("--user={}", mysql_env("MYSQL_USER", "evt")),
        format!("--password={}", mysql_env("MYSQL_PASSWORD", "evt")),
        "--protocol=TCP".to_string(),
    ];
    if let Some(database) = database {
        args.push(format!("-D{database}"));
    }
    args.push("-e".to_string());
    args.push(sql.to_string());
    args
}

fn mysql_root_env(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

fn run_mysql(database: Option<&str>, sql: &str) -> bool {
    Command::new("mysql")
        .args(mysql_args(database, sql))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn run_mysql_admin(sql: &str) -> bool {
    let admin_user = mysql_root_env("MYSQL_ADMIN_USER", "");
    if admin_user.is_empty() {
        return false;
    }

    Command::new("mysql")
        .args([
            format!("--host={}", mysql_root_env("MYSQL_HOST", "127.0.0.1")),
            format!("--port={}", mysql_root_env("MYSQL_PORT", "3306")),
            format!("--user={admin_user}"),
            format!("--password={}", mysql_root_env("MYSQL_ADMIN_PASSWORD", "")),
            "--protocol=TCP".to_string(),
            "-e".to_string(),
            sql.to_string(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn cargo_target_dir() -> PathBuf {
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(target_dir) = current_exe
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
        {
            return target_dir.to_path_buf();
        }
    }

    std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root().join("target"))
}

fn evt_binary_path() -> PathBuf {
    cargo_target_dir().join("debug/evt")
}

fn ensure_evt_binary_built() {
    static BUILD_ONCE: OnceLock<()> = OnceLock::new();
    BUILD_ONCE.get_or_init(|| {
        let status = Command::new("cargo")
            .current_dir(workspace_root())
            .arg("build")
            .arg("-p")
            .arg("evt-app")
            .arg("--target-dir")
            .arg(cargo_target_dir())
            .status()
            .expect("build evt-app binary");
        assert!(status.success(), "cargo build -p evt-app failed");
    });
}

fn e2e_guard() -> &'static Mutex<()> {
    static E2E_GUARD: OnceLock<Mutex<()>> = OnceLock::new();
    E2E_GUARD.get_or_init(|| Mutex::new(()))
}

fn lock_e2e_guard() -> MutexGuard<'static, ()> {
    e2e_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

async fn write_test_web_dist(dir: &PathBuf) {
    fs::create_dir_all(dir.join("assets"))
        .await
        .expect("create web dist assets");
    fs::write(
        dir.join("index.html"),
        "<!doctype html><html><body><div id=\"app\"></div><script type=\"module\" src=\"/assets/app.js\"></script></body></html>",
    )
    .await
    .expect("write index.html");
    fs::write(dir.join("assets/app.js"), "console.log('evt-e2e-asset');")
        .await
        .expect("write app asset");
}

struct LocalServer {
    base_url: String,
    grpc_port: u16,
    child: Child,
    database_name: String,
    isolated_database: bool,
    test_user_ids: Vec<i64>,
}

impl LocalServer {
    async fn start() -> Self {
        ensure_evt_binary_built();

        let suffix = unique_suffix();
        let http_port = reserve_port();
        let grpc_port = reserve_port();
        let database_name = format!("evt_e2e_{suffix}");
        let isolated_database = run_mysql_admin(&format!(
            "DROP DATABASE IF EXISTS `{database_name}`; CREATE DATABASE `{database_name}` CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci;"
        )) || run_mysql(
            None,
            &format!(
                "DROP DATABASE IF EXISTS `{database_name}`; CREATE DATABASE `{database_name}` CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci;"
            ),
        );
        let active_database = if isolated_database {
            database_name.clone()
        } else {
            mysql_env("MYSQL_CLEAN_DATABASE", "evt")
        };
        let _ = run_mysql(
            Some(&active_database),
            "DELETE FROM site_settings WHERE id = 1;",
        );

        let temp_root = std::env::temp_dir().join(format!("evt-http-e2e-{suffix}"));
        let web_dist_dir = temp_root.join("web-dist");
        let storage_dir = temp_root.join("storage");
        write_test_web_dist(&web_dist_dir).await;
        fs::create_dir_all(&storage_dir)
            .await
            .expect("create storage dir");

        let settings = Settings {
            app: AppSettings {
                name: "evt-e2e".into(),
                env: "test".into(),
            },
            server: ServerSettings {
                http: HttpSettings {
                    host: "127.0.0.1".into(),
                    port: http_port,
                },
                grpc: GrpcSettings {
                    host: "127.0.0.1".into(),
                    port: grpc_port,
                },
            },
            database: DatabaseSettings {
                url: mysql_database_url(&active_database),
                max_connections: 5,
            },
            jwt: JwtSettings {
                secret: "evt-e2e-secret".into(),
                issuer: "evt".into(),
                expire_seconds: 86_400,
            },
            storage: StorageSettings {
                local_dir: storage_dir.display().to_string(),
            },
            telemetry: TelemetrySettings {
                enabled: false,
                service_name: "evt-e2e".into(),
                otlp_endpoint: "http://127.0.0.1:4317".into(),
            },
            web: WebSettings {
                dist_dir: web_dist_dir.display().to_string(),
            },
            site: SiteSettings {
                enable_spaces: true,
                default_space_slug: "public".into(),
                allow_user_register: true,
                allow_phone_bind: true,
                use_friendship: true,
                enable_trends_bar: true,
                enable_wallet: false,
                allow_tweet_attachment: true,
                allow_tweet_attachment_price: false,
                allow_tweet_video: true,
                default_tweet_max_length: 500,
                tweet_web_ellipsis_size: 220,
                tweet_mobile_ellipsis_size: 140,
                default_tweet_visibility: "public".into(),
                default_msg_loop_interval: 5_000,
                copyright_top: "Evt".into(),
                copyright_left: "Self-hosted".into(),
                copyright_left_link: "".into(),
                copyright_right: "MIT".into(),
                copyright_right_link: "https://opensource.org/license/mit".into(),
            },
        };

        let settings_path = temp_root.join("settings.toml");
        let log_path = temp_root.join("server.log");
        let settings_text = format!(
            "[app]\nname = \"{}\"\nenv = \"{}\"\n\n[server.http]\nhost = \"{}\"\nport = {}\n\n[server.grpc]\nhost = \"{}\"\nport = {}\n\n[database]\nurl = \"{}\"\nmax_connections = {}\n\n[jwt]\nsecret = \"{}\"\nissuer = \"{}\"\nexpire_seconds = {}\n\n[storage]\nlocal_dir = \"{}\"\n\n[telemetry]\nenabled = {}\nservice_name = \"{}\"\notlp_endpoint = \"{}\"\n\n[web]\ndist_dir = \"{}\"\n\n[site]\nenable_spaces = {}\ndefault_space_slug = \"{}\"\nallow_user_register = {}\nallow_phone_bind = {}\nuse_friendship = {}\nenable_trends_bar = {}\nenable_wallet = {}\nallow_tweet_attachment = {}\nallow_tweet_attachment_price = {}\nallow_tweet_video = {}\ndefault_tweet_max_length = {}\ntweet_web_ellipsis_size = {}\ntweet_mobile_ellipsis_size = {}\ndefault_tweet_visibility = \"{}\"\ndefault_msg_loop_interval = {}\ncopyright_top = \"{}\"\ncopyright_left = \"{}\"\ncopyright_left_link = \"{}\"\ncopyright_right = \"{}\"\ncopyright_right_link = \"{}\"\n",
            settings.app.name,
            settings.app.env,
            settings.server.http.host,
            settings.server.http.port,
            settings.server.grpc.host,
            settings.server.grpc.port,
            settings.database.url,
            settings.database.max_connections,
            settings.jwt.secret,
            settings.jwt.issuer,
            settings.jwt.expire_seconds,
            settings.storage.local_dir,
            settings.telemetry.enabled,
            settings.telemetry.service_name,
            settings.telemetry.otlp_endpoint,
            settings.web.dist_dir,
            settings.site.enable_spaces,
            settings.site.default_space_slug,
            settings.site.allow_user_register,
            settings.site.allow_phone_bind,
            settings.site.use_friendship,
            settings.site.enable_trends_bar,
            settings.site.enable_wallet,
            settings.site.allow_tweet_attachment,
            settings.site.allow_tweet_attachment_price,
            settings.site.allow_tweet_video,
            settings.site.default_tweet_max_length,
            settings.site.tweet_web_ellipsis_size,
            settings.site.tweet_mobile_ellipsis_size,
            settings.site.default_tweet_visibility,
            settings.site.default_msg_loop_interval,
            settings.site.copyright_top,
            settings.site.copyright_left,
            settings.site.copyright_left_link,
            settings.site.copyright_right,
            settings.site.copyright_right_link,
        );
        fs::write(&settings_path, settings_text)
            .await
            .expect("write settings file");

        let stdout = File::create(&log_path).expect("create server log");
        let stderr = stdout.try_clone().expect("clone server log handle");

        let child = Command::new(evt_binary_path())
            .current_dir(workspace_root())
            .env("EVT_RS_CONFIG", settings_path)
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .spawn()
            .expect("spawn evt server");

        let base_url = format!("http://127.0.0.1:{http_port}");
        let client = reqwest::Client::new();
        for _ in 0..60 {
            if let Ok(response) = client.get(format!("{base_url}/healthz")).send().await {
                if response.status().is_success() {
                    return Self {
                        base_url,
                        grpc_port,
                        child,
                        database_name: active_database,
                        isolated_database,
                        test_user_ids: Vec::new(),
                    };
                }
            }
            sleep(Duration::from_millis(250)).await;
        }

        let server_log = std::fs::read_to_string(&log_path).unwrap_or_default();
        panic!("local e2e server did not become healthy\nserver log:\n{server_log}");
    }
}

impl Drop for LocalServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();

        if !self.test_user_ids.is_empty() {
            let cleanup_db = self.database_name.as_str();
            let mut user_ids = self.test_user_ids.clone();
            user_ids.sort_unstable();
            user_ids.dedup();
            let joined = user_ids
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            let _ = run_mysql(
                Some(cleanup_db),
                &format!("DELETE FROM users WHERE id IN ({joined});"),
            );
        }

        if self.isolated_database {
            let _ = run_mysql(
                None,
                &format!("DROP DATABASE IF EXISTS `{}`;", self.database_name),
            );
            let _ = run_mysql_admin(&format!(
                "DROP DATABASE IF EXISTS `{}`;",
                self.database_name
            ));
        }
    }
}

impl LocalServer {
    fn register_test_user(&mut self, user_id: i64) {
        self.test_user_ids.push(user_id);
    }

    fn promote_user_to_admin(&self, user_id: i64) {
        let updated = run_mysql(
            Some(&self.database_name),
            &format!(
                "INSERT INTO user_profiles (user_id, nickname, is_admin) VALUES ({user_id}, '', TRUE) ON DUPLICATE KEY UPDATE is_admin = TRUE;"
            ),
        );
        assert!(updated, "failed to promote test user {user_id} to admin");
    }
}

#[tokio::test]
async fn local_http_e2e_covers_web_and_legacy_post_comment_flow() {
    let _guard = lock_e2e_guard();
    let mut server = LocalServer::start().await;
    let client = reqwest::Client::new();
    let username = format!("evt_e2e_{}", unique_suffix());
    let password = "Passw0rd_123";

    let healthz = client
        .get(format!("{}/healthz", server.base_url))
        .send()
        .await
        .expect("healthz response");
    assert_eq!(healthz.status(), StatusCode::OK);
    let healthz_body: Value = healthz.json().await.expect("healthz json");
    assert_eq!(healthz_body["code"], 0);
    assert_eq!(healthz_body["data"]["status"], "ok");

    let index = client
        .get(&server.base_url)
        .send()
        .await
        .expect("index response");
    assert_eq!(index.status(), StatusCode::OK);
    let index_body = index.text().await.expect("index text");
    assert!(index_body.contains("<div id=\"app\"></div>"));
    assert!(index_body.contains("/assets/app.js"));

    let asset = client
        .get(format!("{}/assets/app.js", server.base_url))
        .send()
        .await
        .expect("asset response");
    assert_eq!(asset.status(), StatusCode::OK);
    assert!(
        asset
            .text()
            .await
            .expect("asset text")
            .contains("evt-e2e-asset")
    );

    let site_profile = client
        .get(format!("{}/v1/site/profile", server.base_url))
        .send()
        .await
        .expect("site profile response");
    assert_eq!(site_profile.status(), StatusCode::OK);
    let site_profile_body: Value = site_profile.json().await.expect("site profile json");
    assert_eq!(site_profile_body["code"], 0);
    assert!(site_profile_body["data"]["allow_user_register"].is_boolean());

    let register = client
        .post(format!("{}/v1/auth/register", server.base_url))
        .json(&serde_json::json!({
            "username": username,
            "password": password,
        }))
        .send()
        .await
        .expect("register response");
    assert_eq!(register.status(), StatusCode::OK);
    let register_body: Value = register.json().await.expect("register json");
    assert_eq!(register_body["code"], 0);

    let login = client
        .post(format!("{}/v1/auth/login", server.base_url))
        .json(&serde_json::json!({
            "username": username,
            "password": password,
        }))
        .send()
        .await
        .expect("login response");
    assert_eq!(login.status(), StatusCode::OK);
    let login_body: Value = login.json().await.expect("login json");
    let token = login_body["data"]["token"]
        .as_str()
        .expect("login token")
        .to_string();

    let user_info = client
        .get(format!("{}/v1/user/info", server.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("legacy user info response");
    assert_eq!(user_info.status(), StatusCode::OK);
    let user_info_body: Value = user_info.json().await.expect("user info json");
    assert_eq!(user_info_body["code"], 0);
    server.register_test_user(
        user_info_body["data"]["id"]
            .as_i64()
            .expect("first registered user id"),
    );

    let current_user = client
        .get(format!("{}/v1/users/me", server.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("current user response");
    assert_eq!(current_user.status(), StatusCode::OK);
    let current_user_body: Value = current_user.json().await.expect("current user json");
    assert_eq!(current_user_body["code"], 0);
    assert_eq!(
        current_user_body["data"]["username"].as_str(),
        Some(username.as_str())
    );
    let first_user_id = current_user_body["data"]["id"]
        .as_i64()
        .expect("first user id");
    if current_user_body["data"]["is_admin"] != true {
        server.promote_user_to_admin(first_user_id);
    }

    let spaces = client
        .get(format!("{}/v1/spaces?limit=20", server.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("list spaces response");
    assert_eq!(spaces.status(), StatusCode::OK);
    let spaces_body: Value = spaces.json().await.expect("list spaces json");
    assert_eq!(spaces_body["code"], 0);
    assert!(
        spaces_body["data"]
            .as_array()
            .expect("spaces list")
            .iter()
            .any(|item| item["slug"].as_str() == Some("public"))
    );

    let create_post_with_legacy_default_slug = client
        .post(format!("{}/v1/post", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "space_slug": "square",
            "contents": [
                { "content": "legacy square alias post", "type": 2, "sort": 100 }
            ],
            "tags": [],
            "users": [],
            "attachment_price": 0,
            "visibility": 0
        }))
        .send()
        .await
        .expect("create post with legacy square slug");
    assert_eq!(
        create_post_with_legacy_default_slug.status(),
        StatusCode::OK
    );

    let create_post_with_runtime_default_space = client
        .post(format!("{}/v1/post", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "contents": [
                { "content": "implicit default public space post", "type": 2, "sort": 100 }
            ],
            "tags": [],
            "users": [],
            "attachment_price": 0,
            "visibility": 0
        }))
        .send()
        .await
        .expect("create post with implicit default space");
    assert_eq!(
        create_post_with_runtime_default_space.status(),
        StatusCode::OK
    );

    let create_private_space = client
        .post(format!("{}/v1/spaces", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "slug": format!("team-{}", unique_suffix()),
            "name": "Team Space",
            "description": "Private team collaboration space",
            "visibility": "private"
        }))
        .send()
        .await
        .expect("create private space response");
    assert_eq!(create_private_space.status(), StatusCode::OK);
    let create_private_space_body: Value = create_private_space
        .json()
        .await
        .expect("create private space json");
    let private_space_slug = create_private_space_body["data"]["slug"]
        .as_str()
        .expect("private space slug")
        .to_string();
    assert_eq!(create_private_space_body["data"]["visibility"], "private");

    let create_post = client
        .post(format!("{}/v1/post", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "space_slug": private_space_slug,
            "contents": [
                { "content": "evt e2e post", "type": 2, "sort": 100 }
            ],
            "tags": [],
            "users": [],
            "attachment_price": 0,
            "visibility": 0
        }))
        .send()
        .await
        .expect("create legacy post response");
    assert_eq!(create_post.status(), StatusCode::OK);
    let create_post_body: Value = create_post.json().await.expect("create post json");
    let post_id = create_post_body["data"]["id"].as_i64().expect("post id");
    assert_eq!(
        create_post_body["data"]["contents"][0]["content"].as_str(),
        Some("evt e2e post")
    );

    let member_username = format!("evt_member_{}", unique_suffix());
    let member_password = "Passw0rd_456";

    let second_register = client
        .post(format!("{}/v1/auth/register", server.base_url))
        .json(&serde_json::json!({
            "username": member_username,
            "password": member_password,
        }))
        .send()
        .await
        .expect("second register response");
    assert_eq!(second_register.status(), StatusCode::OK);

    let second_login = client
        .post(format!("{}/v1/auth/login", server.base_url))
        .json(&serde_json::json!({
            "username": member_username,
            "password": member_password,
        }))
        .send()
        .await
        .expect("second login response");
    assert_eq!(second_login.status(), StatusCode::OK);
    let second_login_body: Value = second_login.json().await.expect("second login json");
    let second_token = second_login_body["data"]["token"]
        .as_str()
        .expect("second login token")
        .to_string();

    let second_current_user = client
        .get(format!("{}/v1/users/me", server.base_url))
        .bearer_auth(&second_token)
        .send()
        .await
        .expect("second current user response");
    assert_eq!(second_current_user.status(), StatusCode::OK);
    let second_current_user_body: Value = second_current_user
        .json()
        .await
        .expect("second current user json");
    let second_user_id = second_current_user_body["data"]["id"]
        .as_i64()
        .expect("second user id");
    server.register_test_user(second_user_id);

    if current_user_body["data"]["is_admin"] != true {
        server.promote_user_to_admin(first_user_id);
    }

    let second_spaces_before = client
        .get(format!("{}/v1/spaces?limit=20", server.base_url))
        .bearer_auth(&second_token)
        .send()
        .await
        .expect("second list spaces before invite");
    assert_eq!(second_spaces_before.status(), StatusCode::OK);
    let second_spaces_before_body: Value = second_spaces_before
        .json()
        .await
        .expect("second list spaces before invite json");
    assert!(
        second_spaces_before_body["data"]
            .as_array()
            .expect("second spaces before invite list")
            .iter()
            .all(|item| item["slug"].as_str() != Some(private_space_slug.as_str()))
    );

    let second_new_user_posts_before_invite = client
        .get(format!(
            "{}/v1/users/{}/posts?page=1&page_size=20",
            server.base_url, username
        ))
        .bearer_auth(&second_token)
        .send()
        .await
        .expect("second new user posts before invite");
    assert_eq!(second_new_user_posts_before_invite.status(), StatusCode::OK);
    let second_new_user_posts_before_invite_body: Value = second_new_user_posts_before_invite
        .json()
        .await
        .expect("second new user posts before invite json");
    assert!(
        second_new_user_posts_before_invite_body["data"]["items"]
            .as_array()
            .expect("second new user posts before invite list")
            .iter()
            .all(|item| item["id"].as_i64() != Some(post_id))
    );

    let second_legacy_user_posts_before_invite = client
        .get(format!(
            "{}/v1/user/posts?username={}&style=post&page=1&page_size=20",
            server.base_url, username
        ))
        .bearer_auth(&second_token)
        .send()
        .await
        .expect("second legacy user posts before invite");
    assert_eq!(
        second_legacy_user_posts_before_invite.status(),
        StatusCode::OK
    );
    let second_legacy_user_posts_before_invite_body: Value = second_legacy_user_posts_before_invite
        .json()
        .await
        .expect("second legacy user posts before invite json");
    assert!(
        second_legacy_user_posts_before_invite_body["data"]["list"]
            .as_array()
            .expect("second legacy user posts before invite list")
            .iter()
            .all(|item| item["id"].as_i64() != Some(post_id))
    );

    let second_get_post_before_invite = client
        .get(format!("{}/v1/post?id={post_id}", server.base_url))
        .bearer_auth(&second_token)
        .send()
        .await
        .expect("second get post before invite");
    assert_eq!(
        second_get_post_before_invite.status(),
        StatusCode::UNAUTHORIZED
    );

    let second_rest_get_post_before_invite = client
        .get(format!("{}/v1/posts/{post_id}", server.base_url))
        .bearer_auth(&second_token)
        .send()
        .await
        .expect("second rest get post before invite");
    assert_eq!(
        second_rest_get_post_before_invite.status(),
        StatusCode::UNAUTHORIZED
    );

    let second_rest_comments_before_invite = client
        .get(format!(
            "{}/v1/posts/{post_id}/comments?page=1&page_size=20",
            server.base_url
        ))
        .bearer_auth(&second_token)
        .send()
        .await
        .expect("second rest comments before invite");
    assert_eq!(
        second_rest_comments_before_invite.status(),
        StatusCode::UNAUTHORIZED
    );

    let second_profile_before_invite = client
        .get(format!(
            "{}/v1/users/{}/profile",
            server.base_url, username
        ))
        .send()
        .await
        .expect("second profile before invite");
    assert_eq!(second_profile_before_invite.status(), StatusCode::OK);
    let second_profile_before_invite_body: Value = second_profile_before_invite
        .json()
        .await
        .expect("second profile before invite json");
    assert_eq!(second_profile_before_invite_body["data"]["posts_count"], 2);
    assert_eq!(second_profile_before_invite_body["data"]["comments_count"], 0);

    let second_legacy_profile_before_invite = client
        .get(format!(
            "{}/v1/user/profile?username={}",
            server.base_url, username
        ))
        .send()
        .await
        .expect("second legacy profile before invite");
    assert_eq!(second_legacy_profile_before_invite.status(), StatusCode::OK);
    let second_legacy_profile_before_invite_body: Value = second_legacy_profile_before_invite
        .json()
        .await
        .expect("second legacy profile before invite json");
    assert_eq!(
        second_legacy_profile_before_invite_body["data"]["tweets_count"],
        2
    );

    let second_post_star_before_invite = client
        .post(format!("{}/v1/post/star", server.base_url))
        .bearer_auth(&second_token)
        .json(&serde_json::json!({
            "id": post_id
        }))
        .send()
        .await
        .expect("second post star before invite");
    assert_eq!(second_post_star_before_invite.status(), StatusCode::UNAUTHORIZED);

    let add_member = client
        .post(format!("{}/v1/spaces/members", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "space_id": create_private_space_body["data"]["id"],
            "username": member_username,
            "role": "admin"
        }))
        .send()
        .await
        .expect("add space member response");
    assert_eq!(add_member.status(), StatusCode::OK);
    let add_member_body: Value = add_member.json().await.expect("add space member json");
    assert_eq!(add_member_body["data"]["username"], member_username);
    assert_eq!(add_member_body["data"]["role"], "admin");

    let members = client
        .get(format!(
            "{}/v1/spaces/members?space_id={}",
            server.base_url, create_private_space_body["data"]["id"]
        ))
        .bearer_auth(&token)
        .send()
        .await
        .expect("list members response");
    assert_eq!(members.status(), StatusCode::OK);
    let members_body: Value = members.json().await.expect("list members json");
    assert!(
        members_body["data"]
            .as_array()
            .expect("members list")
            .iter()
            .any(|item| {
                item["username"].as_str() == Some(member_username.as_str())
                    && item["role"].as_str() == Some("admin")
            })
    );

    let second_spaces_after = client
        .get(format!("{}/v1/spaces?limit=20", server.base_url))
        .bearer_auth(&second_token)
        .send()
        .await
        .expect("second list spaces after invite");
    assert_eq!(second_spaces_after.status(), StatusCode::OK);
    let second_spaces_after_body: Value = second_spaces_after
        .json()
        .await
        .expect("second list spaces after invite json");
    assert!(
        second_spaces_after_body["data"]
            .as_array()
            .expect("second spaces after invite list")
            .iter()
            .any(|item| {
                item["slug"].as_str() == Some(private_space_slug.as_str())
                    && item["current_user_role"].as_str() == Some("admin")
            })
    );

    let second_get_post_after_invite = client
        .get(format!("{}/v1/post?id={post_id}", server.base_url))
        .bearer_auth(&second_token)
        .send()
        .await
        .expect("second get post after invite");
    assert_eq!(second_get_post_after_invite.status(), StatusCode::OK);

    let second_rest_get_post_after_invite = client
        .get(format!("{}/v1/posts/{post_id}", server.base_url))
        .bearer_auth(&second_token)
        .send()
        .await
        .expect("second rest get post after invite");
    assert_eq!(second_rest_get_post_after_invite.status(), StatusCode::OK);

    let second_rest_comments_after_invite = client
        .get(format!(
            "{}/v1/posts/{post_id}/comments?page=1&page_size=20",
            server.base_url
        ))
        .bearer_auth(&second_token)
        .send()
        .await
        .expect("second rest comments after invite");
    assert_eq!(second_rest_comments_after_invite.status(), StatusCode::OK);

    let second_profile_after_invite = client
        .get(format!(
            "{}/v1/users/{}/profile",
            server.base_url, username
        ))
        .bearer_auth(&second_token)
        .send()
        .await
        .expect("second profile after invite");
    assert_eq!(second_profile_after_invite.status(), StatusCode::OK);
    let second_profile_after_invite_body: Value = second_profile_after_invite
        .json()
        .await
        .expect("second profile after invite json");
    assert_eq!(second_profile_after_invite_body["data"]["posts_count"], 3);
    assert_eq!(second_profile_after_invite_body["data"]["comments_count"], 0);

    let second_new_user_posts_after_invite = client
        .get(format!(
            "{}/v1/users/{}/posts?page=1&page_size=20",
            server.base_url, username
        ))
        .bearer_auth(&second_token)
        .send()
        .await
        .expect("second new user posts after invite");
    assert_eq!(second_new_user_posts_after_invite.status(), StatusCode::OK);
    let second_new_user_posts_after_invite_body: Value = second_new_user_posts_after_invite
        .json()
        .await
        .expect("second new user posts after invite json");
    assert!(
        second_new_user_posts_after_invite_body["data"]["items"]
            .as_array()
            .expect("second new user posts after invite list")
            .iter()
            .any(|item| item["id"].as_i64() == Some(post_id))
    );

    let second_legacy_user_posts_after_invite = client
        .get(format!(
            "{}/v1/user/posts?username={}&style=post&page=1&page_size=20",
            server.base_url, username
        ))
        .bearer_auth(&second_token)
        .send()
        .await
        .expect("second legacy user posts after invite");
    assert_eq!(
        second_legacy_user_posts_after_invite.status(),
        StatusCode::OK
    );
    let second_legacy_user_posts_after_invite_body: Value = second_legacy_user_posts_after_invite
        .json()
        .await
        .expect("second legacy user posts after invite json");
    assert!(
        second_legacy_user_posts_after_invite_body["data"]["list"]
            .as_array()
            .expect("second legacy user posts after invite list")
            .iter()
            .any(|item| item["id"].as_i64() == Some(post_id))
    );

    let get_post_star_before = client
        .get(format!("{}/v1/post/star?id={post_id}", server.base_url))
        .bearer_auth(&second_token)
        .send()
        .await
        .expect("get post star before toggle");
    assert_eq!(get_post_star_before.status(), StatusCode::OK);
    let get_post_star_before_body: Value = get_post_star_before
        .json()
        .await
        .expect("post star before json");
    assert_eq!(get_post_star_before_body["data"]["status"], false);

    let toggle_post_star = client
        .post(format!("{}/v1/post/star", server.base_url))
        .bearer_auth(&second_token)
        .json(&serde_json::json!({ "id": post_id }))
        .send()
        .await
        .expect("toggle post star");
    assert_eq!(toggle_post_star.status(), StatusCode::OK);
    let toggle_post_star_body: Value = toggle_post_star
        .json()
        .await
        .expect("toggle post star json");
    assert_eq!(toggle_post_star_body["data"]["status"], true);

    let third_username = format!("evt_observer_{}", unique_suffix());
    let third_password = "Passw0rd_654";

    let third_register = client
        .post(format!("{}/v1/auth/register", server.base_url))
        .json(&serde_json::json!({
            "username": third_username,
            "password": third_password,
        }))
        .send()
        .await
        .expect("third register response");
    assert_eq!(third_register.status(), StatusCode::OK);

    let third_login = client
        .post(format!("{}/v1/auth/login", server.base_url))
        .json(&serde_json::json!({
            "username": third_username,
            "password": third_password,
        }))
        .send()
        .await
        .expect("third login response");
    assert_eq!(third_login.status(), StatusCode::OK);
    let third_login_body: Value = third_login.json().await.expect("third login json");
    let third_token = third_login_body["data"]["token"]
        .as_str()
        .expect("third login token")
        .to_string();

    let third_current_user = client
        .get(format!("{}/v1/users/me", server.base_url))
        .bearer_auth(&third_token)
        .send()
        .await
        .expect("third current user response");
    assert_eq!(third_current_user.status(), StatusCode::OK);
    let third_current_user_body: Value = third_current_user
        .json()
        .await
        .expect("third current user json");
    server.register_test_user(
        third_current_user_body["data"]["id"]
            .as_i64()
            .expect("third user id"),
    );

    let add_third_member = client
        .post(format!("{}/v1/spaces/members", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "space_id": create_private_space_body["data"]["id"],
            "username": third_username,
            "role": "member"
        }))
        .send()
        .await
        .expect("add third space member response");
    assert_eq!(add_third_member.status(), StatusCode::OK);

    let third_get_post_after_invite = client
        .get(format!("{}/v1/post?id={post_id}", server.base_url))
        .bearer_auth(&third_token)
        .send()
        .await
        .expect("third get post after invite");
    assert_eq!(third_get_post_after_invite.status(), StatusCode::OK);

    let remove_third_member = client
        .delete(format!("{}/v1/spaces/members", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "space_id": create_private_space_body["data"]["id"],
            "user_id": third_current_user_body["data"]["id"],
        }))
        .send()
        .await
        .expect("remove third space member response");
    assert_eq!(remove_third_member.status(), StatusCode::OK);

    let get_post = client
        .get(format!("{}/v1/post?id={post_id}", server.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("get legacy post response");
    assert_eq!(get_post.status(), StatusCode::OK);
    let get_post_body: Value = get_post.json().await.expect("get post json");
    assert_eq!(get_post_body["code"], 0);

    let create_comment = client
        .post(format!("{}/v1/post/comment", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "post_id": post_id,
            "contents": [
                { "content": "evt e2e comment", "type": 2, "sort": 100 }
            ],
            "users": []
        }))
        .send()
        .await
        .expect("create legacy comment response");
    assert_eq!(create_comment.status(), StatusCode::OK);
    let create_comment_body: Value = create_comment.json().await.expect("create comment json");
    assert_eq!(create_comment_body["code"], 0);
    assert_eq!(
        create_comment_body["data"]["contents"][0]["content"].as_str(),
        Some("evt e2e comment")
    );
    let comment_id = create_comment_body["data"]["id"]
        .as_i64()
        .expect("comment id");

    let second_owned_post = client
        .post(format!("{}/v1/post", server.base_url))
        .bearer_auth(&second_token)
        .json(&serde_json::json!({
            "space_slug": private_space_slug,
            "contents": [
                { "content": "member-owned private post", "type": 2, "sort": 100 }
            ],
            "tags": [],
            "users": [],
            "attachment_price": 0,
            "visibility": 0
        }))
        .send()
        .await
        .expect("create second owned private post response");
    assert_eq!(second_owned_post.status(), StatusCode::OK);
    let second_owned_post_body: Value = second_owned_post
        .json()
        .await
        .expect("create second owned private post json");
    let second_owned_post_id = second_owned_post_body["data"]["id"]
        .as_i64()
        .expect("second owned post id");

    let owner_comment_on_second_post = client
        .post(format!("{}/v1/post/comment", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "post_id": second_owned_post_id,
            "contents": [
                { "content": "owner comment on member post", "type": 2, "sort": 100 }
            ],
            "users": []
        }))
        .send()
        .await
        .expect("owner comment on second post response");
    assert_eq!(owner_comment_on_second_post.status(), StatusCode::OK);
    let owner_comment_on_second_post_body: Value = owner_comment_on_second_post
        .json()
        .await
        .expect("owner comment on second post json");
    let owner_comment_on_second_post_id = owner_comment_on_second_post_body["data"]["id"]
        .as_i64()
        .expect("owner comment on second post id");

    let remove_second_member = client
        .delete(format!("{}/v1/spaces/members", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "space_id": create_private_space_body["data"]["id"],
            "user_id": second_user_id,
        }))
        .send()
        .await
        .expect("remove second space member response");
    assert_eq!(remove_second_member.status(), StatusCode::OK);

    let second_comment_highlight_after_removal = client
        .post(format!("{}/v1/post/comment/highlight", server.base_url))
        .bearer_auth(&second_token)
        .json(&serde_json::json!({
            "id": owner_comment_on_second_post_id
        }))
        .send()
        .await
        .expect("second comment highlight after removal response");
    assert_eq!(
        second_comment_highlight_after_removal.status(),
        StatusCode::BAD_REQUEST
    );
    let second_comment_highlight_after_removal_body: Value =
        second_comment_highlight_after_removal
            .json()
            .await
            .expect("second comment highlight after removal json");
    assert_eq!(second_comment_highlight_after_removal_body["code"], 20007);

    for (path, body) in [
        (
            "/v1/post/lock",
            serde_json::json!({ "id": second_owned_post_id }),
        ),
        (
            "/v1/post/highlight",
            serde_json::json!({ "id": second_owned_post_id }),
        ),
        (
            "/v1/post/visibility",
            serde_json::json!({ "id": second_owned_post_id, "visibility": 0 }),
        ),
    ] {
        let response = client
            .post(format!("{}{}", server.base_url, path))
            .bearer_auth(&second_token)
            .json(&body)
            .send()
            .await
            .expect("legacy moderation after removal response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "path={path}");
        let response_body: Value = response
            .json()
            .await
            .expect("legacy moderation after removal json");
        assert_eq!(response_body["code"], 20007, "path={path}");
    }

    let re_add_second_member = client
        .post(format!("{}/v1/spaces/members", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "space_id": create_private_space_body["data"]["id"],
            "username": member_username,
            "role": "admin"
        }))
        .send()
        .await
        .expect("re-add second space member response");
    assert_eq!(re_add_second_member.status(), StatusCode::OK);

    let tagged_post = client
        .post(format!("{}/v1/post", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "space_slug": "public",
            "contents": [
                { "content": "evt tagged rust post", "type": 2, "sort": 100 }
            ],
            "tags": ["rust", "evt"],
            "users": [],
            "attachment_price": 0,
            "visibility": 0
        }))
        .send()
        .await
        .expect("create tagged post response");
    assert_eq!(tagged_post.status(), StatusCode::OK);
    let tagged_post_body: Value = tagged_post.json().await.expect("tagged post json");
    let tagged_post_id = tagged_post_body["data"]["id"]
        .as_i64()
        .expect("tagged post id");

    let wrong_comment_thumbsup = client
        .post(format!("{}/v1/tweet/comment/thumbsup", server.base_url))
        .bearer_auth(&second_token)
        .json(&serde_json::json!({
            "tweet_id": tagged_post_id,
            "comment_id": comment_id
        }))
        .send()
        .await
        .expect("wrong comment thumbsup response");
    assert_eq!(wrong_comment_thumbsup.status(), StatusCode::BAD_REQUEST);

    let list_tags = client
        .get(format!(
            "{}/v1/tags?type=hot&num=20&space_slug=public",
            server.base_url
        ))
        .bearer_auth(&second_token)
        .send()
        .await
        .expect("list tags response");
    assert_eq!(list_tags.status(), StatusCode::OK);
    let list_tags_body: Value = list_tags.json().await.expect("list tags json");
    let rust_topic_id = list_tags_body["data"]["topics"]
        .as_array()
        .expect("topics list")
        .iter()
        .find(|item| item["tag"].as_str() == Some("rust"))
        .and_then(|item| item["id"].as_i64())
        .expect("rust topic id");

    let suggest_tags = client
        .get(format!(
            "{}/v1/suggest/tags?k=ru&space_slug=public",
            server.base_url
        ))
        .send()
        .await
        .expect("suggest tags response");
    assert_eq!(suggest_tags.status(), StatusCode::OK);
    let suggest_tags_body: Value = suggest_tags.json().await.expect("suggest tags json");
    assert!(
        suggest_tags_body["data"]["suggest"]
            .as_array()
            .expect("suggest tags list")
            .iter()
            .any(|item| item.as_str() == Some("rust"))
    );

    for path in [
        "/v1/topic/follow",
        "/v1/topic/stick",
        "/v1/topic/pin",
        "/v1/topic/unfollow",
    ] {
        let topic_action = client
            .post(format!("{}{}", server.base_url, path))
            .bearer_auth(&second_token)
            .json(&serde_json::json!({
                "topic_id": rust_topic_id,
                "space_slug": "public"
            }))
            .send()
            .await
            .expect("topic action response");
        assert_eq!(topic_action.status(), StatusCode::OK, "path={path}");
    }

    let upload_attachment = client
        .post(format!("{}/v1/attachment", server.base_url))
        .bearer_auth(&token)
        .multipart(
            multipart::Form::new().text("type", "attachment").part(
                "file",
                multipart::Part::bytes(b"evt attachment bytes".to_vec())
                    .file_name("demo.txt")
                    .mime_str("text/plain")
                    .expect("attachment mime"),
            ),
        )
        .send()
        .await
        .expect("upload attachment response");
    assert_eq!(upload_attachment.status(), StatusCode::OK);
    let upload_attachment_body: Value = upload_attachment
        .json()
        .await
        .expect("upload attachment json");
    let attachment_path = upload_attachment_body["data"]["content"]
        .as_str()
        .expect("attachment content path")
        .to_string();
    assert!(attachment_path.starts_with("/v1/attachments/"));
    let attachment_id = attachment_path
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .expect("attachment id segment")
        .parse::<i64>()
        .expect("attachment id");

    let upload_image = client
        .post(format!("{}/v1/attachment", server.base_url))
        .bearer_auth(&token)
        .multipart(
            multipart::Form::new().text("type", "public/image").part(
                "file",
                multipart::Part::bytes(
                    b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x02\x00\x00\x00\x90wS\xde\x00\x00\x00\x0cIDAT\x08\x99c```\x00\x00\x00\x04\x00\x01\xf6\x179U\x00\x00\x00\x00IEND\xaeB`\x82".to_vec(),
                )
                .file_name("pixel.png")
                .mime_str("image/png")
                .expect("image mime"),
            ),
        )
        .send()
        .await
        .expect("upload image response");
    assert_eq!(upload_image.status(), StatusCode::OK);
    let upload_image_body: Value = upload_image.json().await.expect("upload image json");
    let image_path = upload_image_body["data"]["content"]
        .as_str()
        .expect("image content path")
        .to_string();
    assert!(image_path.starts_with("/v1/media/"));
    let image_preview = client
        .get(format!("{}{}", server.base_url, image_path))
        .send()
        .await
        .expect("image preview response");
    assert_eq!(image_preview.status(), StatusCode::OK);
    assert_eq!(
        image_preview
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("image/png")
    );
    let image_attachment_id = image_path
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .expect("image attachment id segment")
        .parse::<i64>()
        .expect("image attachment id");
    let legacy_image_preview = client
        .get(format!(
            "{}/v1/attachments/{image_attachment_id}?x-oss-process=image/resize,m_fill,w_300,h_300,limit_0/auto-orient,1/format,webp",
            server.base_url
        ))
        .send()
        .await
        .expect("legacy image preview response");
    assert_eq!(legacy_image_preview.status(), StatusCode::OK);
    assert_eq!(
        legacy_image_preview
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("image/png")
    );
    assert_eq!(
        legacy_image_preview
            .headers()
            .get(reqwest::header::CONTENT_DISPOSITION)
            .and_then(|value| value.to_str().ok()),
        Some("inline; filename=\"pixel.png\"")
    );

    let attachment_post = client
        .post(format!("{}/v1/post", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "space_slug": "public",
            "contents": [
                { "content": "evt attachment post", "type": 2, "sort": 100 },
                { "content": attachment_path, "type": 7, "sort": 101 }
            ],
            "tags": [],
            "users": [],
            "attachment_price": 0,
            "visibility": 0
        }))
        .send()
        .await
        .expect("create attachment post response");
    assert_eq!(attachment_post.status(), StatusCode::OK);
    let attachment_post_body: Value = attachment_post
        .json()
        .await
        .expect("create attachment post json");
    let attachment_content_id = attachment_post_body["data"]["contents"]
        .as_array()
        .expect("attachment post contents")
        .iter()
        .find(|item| item["type"].as_i64() == Some(7))
        .and_then(|item| item["id"].as_i64())
        .expect("attachment content id");

    let attachment_precheck = client
        .get(format!(
            "{}/v1/attachment/precheck?id={attachment_content_id}",
            server.base_url
        ))
        .bearer_auth(&token)
        .send()
        .await
        .expect("attachment precheck response");
    assert_eq!(attachment_precheck.status(), StatusCode::OK);
    let attachment_precheck_body: Value = attachment_precheck
        .json()
        .await
        .expect("attachment precheck json");
    assert_eq!(attachment_precheck_body["data"]["paid"], true);

    let attachment_ticket = client
        .get(format!(
            "{}/v1/attachment?id={attachment_content_id}",
            server.base_url
        ))
        .bearer_auth(&token)
        .send()
        .await
        .expect("attachment ticket response");
    assert_eq!(attachment_ticket.status(), StatusCode::OK);
    let attachment_ticket_body: Value = attachment_ticket
        .json()
        .await
        .expect("attachment ticket json");
    let signed_url = attachment_ticket_body["data"]["signed_url"]
        .as_str()
        .expect("signed url");

    let attachment_download = client
        .get(format!("{}{}", server.base_url, signed_url))
        .send()
        .await
        .expect("attachment download response");
    assert_eq!(attachment_download.status(), StatusCode::OK);
    assert_eq!(
        attachment_download
            .bytes()
            .await
            .expect("attachment bytes")
            .as_ref(),
        b"evt attachment bytes"
    );

    let rest_attachment_download_unauthorized = client
        .get(format!(
            "{}/v1/attachments/{attachment_id}",
            server.base_url
        ))
        .send()
        .await
        .expect("rest attachment unauthorized response");
    assert_eq!(
        rest_attachment_download_unauthorized.status(),
        StatusCode::UNAUTHORIZED
    );

    let rest_attachment_download = client
        .get(format!(
            "{}/v1/attachments/{attachment_id}",
            server.base_url
        ))
        .bearer_auth(&token)
        .send()
        .await
        .expect("rest attachment authorized response");
    assert_eq!(rest_attachment_download.status(), StatusCode::OK);
    assert_eq!(
        rest_attachment_download
            .headers()
            .get(reqwest::header::CONTENT_DISPOSITION)
            .and_then(|value| value.to_str().ok()),
        Some("attachment; filename=\"demo.txt\"")
    );
    assert_eq!(
        rest_attachment_download
            .bytes()
            .await
            .expect("rest attachment bytes")
            .as_ref(),
        b"evt attachment bytes"
    );

    let change_nickname = client
        .post(format!("{}/v1/user/nickname", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "nickname": "EvtAdmin" }))
        .send()
        .await
        .expect("change nickname response");
    assert_eq!(change_nickname.status(), StatusCode::OK);

    let change_avatar = client
        .post(format!("{}/v1/user/avatar", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "avatar": "https://cdn.example.com/avatar.png" }))
        .send()
        .await
        .expect("change avatar response");
    assert_eq!(change_avatar.status(), StatusCode::OK);

    let new_password = "Passw0rd_789";
    let change_password = client
        .post(format!("{}/v1/user/password", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "old_password": password,
            "password": new_password
        }))
        .send()
        .await
        .expect("change password response");
    assert_eq!(change_password.status(), StatusCode::OK);

    let relogin = client
        .post(format!("{}/v1/auth/login", server.base_url))
        .json(&serde_json::json!({
            "username": username,
            "password": new_password,
        }))
        .send()
        .await
        .expect("relogin response");
    assert_eq!(relogin.status(), StatusCode::OK);

    let admin_site_status = client
        .get(format!("{}/v1/admin/site/status", server.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("admin site status response");
    assert_eq!(admin_site_status.status(), StatusCode::OK);
    let admin_site_status_body: Value = admin_site_status
        .json()
        .await
        .expect("admin site status json");
    assert!(
        admin_site_status_body["data"]["register_user_count"]
            .as_i64()
            .expect("register user count")
            >= 2
    );

    let admin_settings_schema = client
        .get(format!("{}/v1/admin/settings/schema", server.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("admin settings schema response");
    assert_eq!(admin_settings_schema.status(), StatusCode::OK);
    let admin_settings_schema_body: Value = admin_settings_schema
        .json()
        .await
        .expect("admin settings schema json");
    assert!(
        admin_settings_schema_body["data"]["items"]
            .as_array()
            .expect("admin settings schema items")
            .iter()
            .any(|item| item["key"].as_str() == Some("web_profile.enable_spaces"))
    );
    assert!(
        admin_settings_schema_body["data"]["items"]
            .as_array()
            .expect("admin settings schema items")
            .iter()
            .any(|item| item["key"].as_str() == Some("web_profile.default_space_slug"))
    );
    assert!(
        admin_settings_schema_body["data"]["items"]
            .as_array()
            .expect("admin settings schema items")
            .iter()
            .any(|item| item["key"].as_str() == Some("web_profile.allow_user_register"))
    );
    assert!(
        admin_settings_schema_body["data"]["items"]
            .as_array()
            .expect("admin settings schema items")
            .iter()
            .any(|item| item["key"].as_str() == Some("web_profile.allow_phone_bind"))
    );
    assert!(
        admin_settings_schema_body["data"]["items"]
            .as_array()
            .expect("admin settings schema items")
            .iter()
            .any(|item| item["key"].as_str() == Some("web_profile.enable_trends_bar"))
    );

    let admin_settings_values = client
        .get(format!("{}/v1/admin/settings/values", server.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("admin settings values response");
    assert_eq!(admin_settings_values.status(), StatusCode::OK);
    let admin_settings_values_body: Value = admin_settings_values
        .json()
        .await
        .expect("admin settings values json");
    assert!(
        admin_settings_values_body["data"]["items"]
            .as_array()
            .expect("admin settings values items")
            .iter()
            .any(|item| item["key"].as_str() == Some("web_profile.enable_spaces"))
    );
    assert!(
        admin_settings_values_body["data"]["items"]
            .as_array()
            .expect("admin settings values items")
            .iter()
            .any(|item| item["key"].as_str() == Some("web_profile.default_space_slug"))
    );
    assert!(
        admin_settings_values_body["data"]["items"]
            .as_array()
            .expect("admin settings values items")
            .iter()
            .any(|item| item["key"].as_str() == Some("web_profile.allow_user_register"))
    );
    assert!(
        admin_settings_values_body["data"]["items"]
            .as_array()
            .expect("admin settings values items")
            .iter()
            .any(|item| item["key"].as_str() == Some("web_profile.allow_phone_bind"))
    );
    assert!(
        admin_settings_values_body["data"]["items"]
            .as_array()
            .expect("admin settings values items")
            .iter()
            .any(|item| item["key"].as_str() == Some("web_profile.enable_trends_bar"))
    );

    let admin_settings_save = client
        .post(format!("{}/v1/admin/settings/save", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "items": [
                { "key": "web_profile.enable_spaces", "value": true },
                { "key": "web_profile.default_space_slug", "value": "public" },
                { "key": "web_profile.allow_user_register", "value": false },
                { "key": "web_profile.allow_phone_bind", "value": false },
                { "key": "web_profile.enable_trends_bar", "value": false },
                { "key": "web_profile.copyright_top", "value": "Evt QA" }
            ]
        }))
        .send()
        .await
        .expect("admin settings save response");
    assert_eq!(admin_settings_save.status(), StatusCode::OK);
    let admin_settings_save_body: Value = admin_settings_save
        .json()
        .await
        .expect("admin settings save json");
    assert!(
        admin_settings_save_body["data"]["updated_keys"]
            .as_array()
            .expect("updated keys")
            .iter()
            .any(|item| item.as_str() == Some("web_profile.allow_user_register"))
    );
    assert!(
        admin_settings_save_body["data"]["updated_keys"]
            .as_array()
            .expect("updated keys")
            .iter()
            .any(|item| item.as_str() == Some("web_profile.enable_trends_bar"))
    );

    let updated_site_profile = client
        .get(format!("{}/v1/site/profile", server.base_url))
        .send()
        .await
        .expect("updated site profile response");
    assert_eq!(updated_site_profile.status(), StatusCode::OK);
    let updated_site_profile_body: Value = updated_site_profile
        .json()
        .await
        .expect("updated site profile json");
    assert_eq!(updated_site_profile_body["data"]["enable_spaces"], true);
    assert_eq!(updated_site_profile_body["data"]["default_space_slug"], "public");
    assert_eq!(updated_site_profile_body["data"]["allow_user_register"], false);
    assert_eq!(updated_site_profile_body["data"]["allow_phone_bind"], false);
    assert_eq!(
        updated_site_profile_body["data"]["enable_trends_bar"],
        false
    );
    assert_eq!(updated_site_profile_body["data"]["copyright_top"], "Evt QA");

    let unread_before_whisper = client
        .get(format!("{}/v1/user/msgcount/unread", server.base_url))
        .bearer_auth(&second_token)
        .send()
        .await
        .expect("unread before whisper response");
    assert_eq!(unread_before_whisper.status(), StatusCode::OK);
    let unread_before_whisper_body: Value = unread_before_whisper
        .json()
        .await
        .expect("unread before whisper json");
    let unread_before_whisper_count = unread_before_whisper_body["data"]["count"]
        .as_i64()
        .expect("unread before whisper count");

    let whisper = client
        .post(format!("{}/v1/user/whisper", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "user_id": second_user_id,
            "content": "secret hello from admin"
        }))
        .send()
        .await
        .expect("whisper response");
    assert_eq!(whisper.status(), StatusCode::OK);

    let unread_after_whisper = client
        .get(format!("{}/v1/user/msgcount/unread", server.base_url))
        .bearer_auth(&second_token)
        .send()
        .await
        .expect("unread after whisper response");
    assert_eq!(unread_after_whisper.status(), StatusCode::OK);
    let unread_after_whisper_body: Value = unread_after_whisper
        .json()
        .await
        .expect("unread after whisper json");
    assert_eq!(
        unread_after_whisper_body["data"]["count"].as_i64(),
        Some(unread_before_whisper_count + 1)
    );

    let whisper_messages = client
        .get(format!(
            "{}/v1/user/messages?style=whisper&page=1&page_size=20",
            server.base_url
        ))
        .bearer_auth(&second_token)
        .send()
        .await
        .expect("whisper messages response");
    assert_eq!(whisper_messages.status(), StatusCode::OK);
    let whisper_messages_body: Value = whisper_messages
        .json()
        .await
        .expect("whisper messages json");
    let whisper_message = whisper_messages_body["data"]["list"]
        .as_array()
        .expect("whisper messages list")
        .iter()
        .find(|item| {
            item["sender_user_id"].as_i64() == Some(first_user_id)
                && item["receiver_user_id"].as_i64() == Some(second_user_id)
                && item["content"].as_str() == Some("secret hello from admin")
        })
        .expect("created whisper message");
    let whisper_message_id = whisper_message["id"].as_i64().expect("whisper message id");

    let read_whisper = client
        .post(format!("{}/v1/user/message/read", server.base_url))
        .bearer_auth(&second_token)
        .json(&serde_json::json!({ "id": whisper_message_id }))
        .send()
        .await
        .expect("read whisper response");
    assert_eq!(read_whisper.status(), StatusCode::OK);

    let unread_after_read = client
        .get(format!("{}/v1/user/msgcount/unread", server.base_url))
        .bearer_auth(&second_token)
        .send()
        .await
        .expect("unread after read response");
    assert_eq!(unread_after_read.status(), StatusCode::OK);
    let unread_after_read_body: Value = unread_after_read
        .json()
        .await
        .expect("unread after read json");
    let unread_after_read_count = unread_after_read_body["data"]["count"]
        .as_i64()
        .expect("unread after read count");
    assert_eq!(
        unread_after_read_count,
        unread_before_whisper_count
    );

    let friend_request = client
        .post(format!("{}/v1/friend/requesting", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "user_id": second_user_id,
            "greetings": "hi evt"
        }))
        .send()
        .await
        .expect("friend request response");
    assert_eq!(friend_request.status(), StatusCode::OK);

    let unread_after_friend_request = client
        .get(format!("{}/v1/user/msgcount/unread", server.base_url))
        .bearer_auth(&second_token)
        .send()
        .await
        .expect("unread after friend request response");
    assert_eq!(unread_after_friend_request.status(), StatusCode::OK);
    let unread_after_friend_request_body: Value = unread_after_friend_request
        .json()
        .await
        .expect("unread after friend request json");
    assert_eq!(
        unread_after_friend_request_body["data"]["count"].as_i64(),
        Some(unread_after_read_count + 1)
    );

    let requesting_messages = client
        .get(format!(
            "{}/v1/user/messages?style=requesting&page=1&page_size=20",
            server.base_url
        ))
        .bearer_auth(&second_token)
        .send()
        .await
        .expect("requesting messages response");
    assert_eq!(requesting_messages.status(), StatusCode::OK);
    let requesting_messages_body: Value = requesting_messages
        .json()
        .await
        .expect("requesting messages json");
    let requesting_message = requesting_messages_body["data"]["list"]
        .as_array()
        .expect("requesting messages list")
        .iter()
        .find(|item| {
            item["sender_user_id"].as_i64() == Some(first_user_id)
                && item["receiver_user_id"].as_i64() == Some(second_user_id)
                && item["content"].as_str() == Some("hi evt")
        })
        .expect("friend request message");
    assert_eq!(requesting_message["type"], 5);
    assert_eq!(requesting_message["reply_id"], 1);

    let all_messages = client
        .get(format!(
            "{}/v1/user/messages?style=all&page=1&page_size=20",
            server.base_url
        ))
        .bearer_auth(&second_token)
        .send()
        .await
        .expect("all messages response");
    assert_eq!(all_messages.status(), StatusCode::OK);
    let all_messages_body: Value = all_messages.json().await.expect("all messages json");
    assert!(
        all_messages_body["data"]["list"]
            .as_array()
            .expect("all messages list")
            .iter()
            .any(|item| item["content"].as_str() == Some("secret hello from admin"))
    );

    let friend_add = client
        .post(format!("{}/v1/friend/add", server.base_url))
        .bearer_auth(&second_token)
        .json(&serde_json::json!({
            "user_id": first_user_id
        }))
        .send()
        .await
        .expect("friend add response");
    assert_eq!(friend_add.status(), StatusCode::OK);

    let requesting_messages_after_add = client
        .get(format!(
            "{}/v1/user/messages?style=requesting&page=1&page_size=20",
            server.base_url
        ))
        .bearer_auth(&second_token)
        .send()
        .await
        .expect("requesting messages after add response");
    assert_eq!(requesting_messages_after_add.status(), StatusCode::OK);
    let requesting_messages_after_add_body: Value = requesting_messages_after_add
        .json()
        .await
        .expect("requesting messages after add json");
    assert!(
        requesting_messages_after_add_body["data"]["list"]
            .as_array()
            .expect("requesting messages after add list")
            .iter()
            .any(|item| {
                item["sender_user_id"].as_i64() == Some(first_user_id)
                    && item["receiver_user_id"].as_i64() == Some(second_user_id)
                    && item["reply_id"].as_i64() == Some(2)
            })
    );

    let second_contacts = client
        .get(format!(
            "{}/v1/user/contacts?page=1&page_size=20",
            server.base_url
        ))
        .bearer_auth(&second_token)
        .send()
        .await
        .expect("second contacts response");
    assert_eq!(second_contacts.status(), StatusCode::OK);
    let second_contacts_body: Value = second_contacts.json().await.expect("second contacts json");
    assert!(
        second_contacts_body["data"]["list"]
            .as_array()
            .expect("second contacts list")
            .iter()
            .any(|item| item["user_id"].as_i64() == Some(first_user_id))
    );

    let first_contacts = client
        .get(format!(
            "{}/v1/user/contacts?page=1&page_size=20",
            server.base_url
        ))
        .bearer_auth(&token)
        .send()
        .await
        .expect("first contacts response");
    assert_eq!(first_contacts.status(), StatusCode::OK);
    let first_contacts_body: Value = first_contacts.json().await.expect("first contacts json");
    assert!(
        first_contacts_body["data"]["list"]
            .as_array()
            .expect("first contacts list")
            .iter()
            .any(|item| item["user_id"].as_i64() == Some(second_user_id))
    );

    let read_all_messages = client
        .post(format!("{}/v1/user/message/readall", server.base_url))
        .bearer_auth(&second_token)
        .send()
        .await
        .expect("read all messages response");
    assert_eq!(read_all_messages.status(), StatusCode::OK);

    let unread_after_read_all = client
        .get(format!("{}/v1/user/msgcount/unread", server.base_url))
        .bearer_auth(&second_token)
        .send()
        .await
        .expect("unread after read all response");
    assert_eq!(unread_after_read_all.status(), StatusCode::OK);
    let unread_after_read_all_body: Value = unread_after_read_all
        .json()
        .await
        .expect("unread after read all json");
    assert_eq!(unread_after_read_all_body["data"]["count"], 0);

    for style in ["default", "hots", "newest"] {
        let list_comments = client
            .get(format!(
                "{}/v1/post/comments?id={post_id}&style={style}&page=1&page_size=20",
                server.base_url
            ))
            .bearer_auth(&token)
            .send()
            .await
            .expect("list legacy comments response");
        assert_eq!(list_comments.status(), StatusCode::OK, "style={style}");
        let list_comments_body: Value = list_comments.json().await.expect("list comments json");
        assert_eq!(list_comments_body["code"], 0, "style={style}");
        assert!(
            list_comments_body["data"]["list"]
                .as_array()
                .expect("legacy comments list")
                .iter()
                .any(|item| item["post_id"].as_i64() == Some(post_id)),
            "style={style} body={list_comments_body}"
        );
    }

    let list_posts = client
        .get(format!("{}/v1/posts?page=1&page_size=20", server.base_url))
        .send()
        .await
        .expect("list posts response");
    assert_eq!(list_posts.status(), StatusCode::OK);
    let list_posts_body: Value = list_posts.json().await.expect("list posts json");
    assert_eq!(list_posts_body["code"], 0);
    assert!(list_posts_body["data"]["list"].is_array());

    let list_posts_with_legacy_space_alias = client
        .get(format!(
            "{}/v1/posts?page=1&page_size=20&space_slug=square",
            server.base_url
        ))
        .send()
        .await
        .expect("list posts with legacy space alias response");
    assert_eq!(list_posts_with_legacy_space_alias.status(), StatusCode::OK);
    let list_posts_with_legacy_space_alias_body: Value = list_posts_with_legacy_space_alias
        .json()
        .await
        .expect("list posts with legacy space alias json");
    assert_eq!(list_posts_with_legacy_space_alias_body["code"], 0);
}

#[tokio::test]
async fn local_http_e2e_falls_back_when_runtime_default_space_slug_is_missing() {
    let _guard = lock_e2e_guard();
    let mut server = LocalServer::start().await;
    let client = reqwest::Client::new();
    let username = format!("evt_space_fallback_{}", unique_suffix());
    let password = "Passw0rd_123";

    let register = client
        .post(format!("{}/v1/auth/register", server.base_url))
        .json(&serde_json::json!({
            "username": username,
            "password": password,
        }))
        .send()
        .await
        .expect("register fallback user response");
    assert_eq!(register.status(), StatusCode::OK);

    let login = client
        .post(format!("{}/v1/auth/login", server.base_url))
        .json(&serde_json::json!({
            "username": username,
            "password": password,
        }))
        .send()
        .await
        .expect("login fallback user response");
    assert_eq!(login.status(), StatusCode::OK);
    let login_body: Value = login.json().await.expect("login fallback user json");
    let token = login_body["data"]["token"]
        .as_str()
        .expect("fallback login token")
        .to_string();

    let current_user = client
        .get(format!("{}/v1/users/me", server.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("fallback current user response");
    assert_eq!(current_user.status(), StatusCode::OK);
    let current_user_body: Value = current_user
        .json()
        .await
        .expect("fallback current user json");
    let user_id = current_user_body["data"]["id"]
        .as_i64()
        .expect("fallback user id");
    server.register_test_user(user_id);

    server.promote_user_to_admin(user_id);

    let save_settings = client
        .post(format!("{}/v1/admin/settings/save", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "items": [
                { "key": "web_profile.default_space_slug", "value": "ghost-space" }
            ]
        }))
        .send()
        .await
        .expect("save fallback default space response");
    assert_eq!(save_settings.status(), StatusCode::OK);

    let site_profile = client
        .get(format!("{}/v1/site/profile", server.base_url))
        .send()
        .await
        .expect("fallback site profile response");
    assert_eq!(site_profile.status(), StatusCode::OK);
    let site_profile_body: Value = site_profile
        .json()
        .await
        .expect("fallback site profile json");
    assert_eq!(site_profile_body["data"]["default_space_slug"], "ghost-space");

    let create_post_without_space = client
        .post(format!("{}/v1/post", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "contents": [
                { "content": "fallback default space post", "type": 2, "sort": 100 }
            ],
            "tags": [],
            "users": [],
            "attachment_price": 0,
            "visibility": 0
        }))
        .send()
        .await
        .expect("create fallback default space post response");
    assert_eq!(create_post_without_space.status(), StatusCode::OK);
    let create_post_without_space_body: Value = create_post_without_space
        .json()
        .await
        .expect("create fallback default space post json");
    let fallback_post_id = create_post_without_space_body["data"]["id"]
        .as_i64()
        .expect("fallback post id");

    let get_post = client
        .get(format!("{}/v1/post?id={fallback_post_id}", server.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("get fallback post response");
    assert_eq!(get_post.status(), StatusCode::OK);
    let get_post_body: Value = get_post.json().await.expect("get fallback post json");
    assert_eq!(get_post_body["data"]["id"].as_i64(), Some(fallback_post_id));

    let list_posts = client
        .get(format!("{}/v1/posts?page=1&page_size=20", server.base_url))
        .send()
        .await
        .expect("list fallback posts response");
    assert_eq!(list_posts.status(), StatusCode::OK);
    let list_posts_body: Value = list_posts
        .json()
        .await
        .expect("list fallback posts json");
    assert!(
        list_posts_body["data"]["list"]
            .as_array()
            .expect("fallback posts list")
            .iter()
            .any(|item| item["id"].as_i64() == Some(fallback_post_id)),
        "body={list_posts_body}"
    );
}

#[tokio::test]
async fn local_http_e2e_first_registered_user_becomes_admin_and_public_space_owner() {
    let _guard = lock_e2e_guard();
    let mut server = LocalServer::start().await;
    if !server.isolated_database {
        eprintln!("skip strict first-user admin assertion without isolated database support");
        return;
    }
    let client = reqwest::Client::new();
    let username = format!("evt_admin_{}", unique_suffix());
    let password = "Passw0rd_123";

    let register = client
        .post(format!("{}/v1/auth/register", server.base_url))
        .json(&serde_json::json!({
            "username": username,
            "password": password,
        }))
        .send()
        .await
        .expect("register first user");
    assert_eq!(register.status(), StatusCode::OK);

    let login = client
        .post(format!("{}/v1/auth/login", server.base_url))
        .json(&serde_json::json!({
            "username": username,
            "password": password,
        }))
        .send()
        .await
        .expect("login first user");
    assert_eq!(login.status(), StatusCode::OK);
    let login_body: Value = login.json().await.expect("login json");
    let token = login_body["data"]["token"]
        .as_str()
        .expect("login token")
        .to_string();

    let current_user = client
        .get(format!("{}/v1/users/me", server.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("current user response");
    assert_eq!(current_user.status(), StatusCode::OK);
    let current_user_body: Value = current_user.json().await.expect("current user json");
    let user_id = current_user_body["data"]["id"]
        .as_i64()
        .expect("current user id");
    server.register_test_user(user_id);
    assert_eq!(current_user_body["data"]["is_admin"], true);

    let spaces = client
        .get(format!("{}/v1/spaces?limit=20", server.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("list spaces");
    assert_eq!(spaces.status(), StatusCode::OK);
    let spaces_body: Value = spaces.json().await.expect("spaces json");
    let public_space = spaces_body["data"]
        .as_array()
        .expect("spaces array")
        .iter()
        .find(|item| item["slug"].as_str() == Some("public"))
        .expect("public space");
    let public_space_id = public_space["id"].as_i64().expect("public space id");
    assert_eq!(public_space["owner_user_id"].as_i64(), Some(user_id));
    assert_eq!(public_space["current_user_role"].as_str(), Some("owner"));

    let list_members = client
        .get(format!(
            "{}/v1/spaces/members?space_id={public_space_id}",
            server.base_url
        ))
        .bearer_auth(&token)
        .send()
        .await
        .expect("public space members response");
    assert_eq!(list_members.status(), StatusCode::OK);
    let list_members_body: Value = list_members
        .json()
        .await
        .expect("public space members json");
    assert!(
        list_members_body["data"]
            .as_array()
            .expect("public space members")
            .iter()
            .any(|item| {
                item["user_id"].as_i64() == Some(user_id)
                    && item["role"].as_str() == Some("owner")
            })
    );

    let admin_site_status = client
        .get(format!("{}/v1/admin/site/status", server.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("admin site status");
    assert_eq!(admin_site_status.status(), StatusCode::OK);
}

#[tokio::test]
async fn local_http_e2e_post_reactions_persist_and_toggle_without_touching_comment_count() {
    let _guard = lock_e2e_guard();
    let mut server = LocalServer::start().await;
    let client = reqwest::Client::new();
    let username = format!("evt_reaction_{}", unique_suffix());
    let password = "Passw0rd_123";

    let register = client
        .post(format!("{}/v1/auth/register", server.base_url))
        .json(&serde_json::json!({
            "username": username,
            "password": password,
        }))
        .send()
        .await
        .expect("register response");
    assert_eq!(register.status(), StatusCode::OK);

    let login = client
        .post(format!("{}/v1/auth/login", server.base_url))
        .json(&serde_json::json!({
            "username": username,
            "password": password,
        }))
        .send()
        .await
        .expect("login response");
    assert_eq!(login.status(), StatusCode::OK);
    let login_body: Value = login.json().await.expect("login json");
    let token = login_body["data"]["token"]
        .as_str()
        .expect("login token")
        .to_string();

    let current_user = client
        .get(format!("{}/v1/users/me", server.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("current user response");
    assert_eq!(current_user.status(), StatusCode::OK);
    let current_user_body: Value = current_user.json().await.expect("current user json");
    let user_id = current_user_body["data"]["id"]
        .as_i64()
        .expect("current user id");
    server.register_test_user(user_id);

    let create_post = client
        .post(format!("{}/v1/post", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "contents": [
                { "content": "reaction target post", "type": 2, "sort": 100 }
            ],
            "tags": [],
            "users": [],
            "attachment_price": 0,
            "visibility": 0
        }))
        .send()
        .await
        .expect("create post response");
    assert_eq!(create_post.status(), StatusCode::OK);
    let create_post_body: Value = create_post.json().await.expect("create post json");
    let post_id = create_post_body["data"]["id"].as_i64().expect("post id");

    let create_comment = client
        .post(format!("{}/v1/posts/{post_id}/comments", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "content": "real comment before reactions"
        }))
        .send()
        .await
        .expect("create comment before reactions");
    assert_eq!(create_comment.status(), StatusCode::OK);

    let initial_reactions = client
        .get(format!("{}/v1/posts/{post_id}/reactions", server.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("initial reactions response");
    assert_eq!(initial_reactions.status(), StatusCode::OK);
    let initial_reactions_body: Value = initial_reactions
        .json()
        .await
        .expect("initial reactions json");
    assert_eq!(initial_reactions_body["data"], serde_json::json!([]));

    let toggle_on = client
        .post(format!("{}/v1/posts/{post_id}/reactions", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "emoji": "😀"
        }))
        .send()
        .await
        .expect("toggle reaction on");
    assert_eq!(toggle_on.status(), StatusCode::OK);
    let toggle_on_body: Value = toggle_on.json().await.expect("toggle on json");
    assert_eq!(toggle_on_body["data"]["active"], true);
    assert_eq!(toggle_on_body["data"]["comment_count"], 1);
    assert!(
        toggle_on_body["data"]["reactions"]
            .as_array()
            .expect("reactions array")
            .iter()
            .any(|item| {
                item["emoji"].as_str() == Some("😀")
                    && item["count"].as_i64() == Some(1)
                    && item["active"] == true
            })
    );

    let persisted_reactions = client
        .get(format!("{}/v1/posts/{post_id}/reactions", server.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("persisted reactions response");
    assert_eq!(persisted_reactions.status(), StatusCode::OK);
    let persisted_reactions_body: Value = persisted_reactions
        .json()
        .await
        .expect("persisted reactions json");
    assert!(
        persisted_reactions_body["data"]
            .as_array()
            .expect("persisted reactions array")
            .iter()
            .any(|item| {
                item["emoji"].as_str() == Some("😀")
                    && item["count"].as_i64() == Some(1)
                    && item["active"] == true
            })
    );

    let get_post_after_reaction = client
        .get(format!("{}/v1/posts/{post_id}", server.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("get post after reaction");
    assert_eq!(get_post_after_reaction.status(), StatusCode::OK);
    let get_post_after_reaction_body: Value = get_post_after_reaction
        .json()
        .await
        .expect("get post after reaction json");
    assert_eq!(get_post_after_reaction_body["data"]["comments_count"], 1);

    let toggle_off = client
        .post(format!("{}/v1/posts/{post_id}/reactions", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "emoji": "😀"
        }))
        .send()
        .await
        .expect("toggle reaction off");
    assert_eq!(toggle_off.status(), StatusCode::OK);
    let toggle_off_body: Value = toggle_off.json().await.expect("toggle off json");
    assert_eq!(toggle_off_body["data"]["active"], false);
    assert_eq!(toggle_off_body["data"]["comment_count"], 1);
    assert!(
        toggle_off_body["data"]["reactions"]
            .as_array()
            .expect("reactions array after toggle off")
            .iter()
            .all(|item| item["emoji"].as_str() != Some("😀") || item["count"].as_i64() == Some(0))
    );

    let reactions_after_toggle_off = client
        .get(format!("{}/v1/posts/{post_id}/reactions", server.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("reactions after toggle off");
    assert_eq!(reactions_after_toggle_off.status(), StatusCode::OK);
    let reactions_after_toggle_off_body: Value = reactions_after_toggle_off
        .json()
        .await
        .expect("reactions after toggle off json");
    assert!(
        reactions_after_toggle_off_body["data"]
            .as_array()
            .expect("reactions after toggle off array")
            .iter()
            .all(|item| item["emoji"].as_str() != Some("😀"))
    );
}

#[tokio::test]
async fn local_http_e2e_admin_settings_disable_registration_takes_effect_immediately() {
    let _guard = lock_e2e_guard();
    let mut server = LocalServer::start().await;
    if !server.isolated_database {
        eprintln!("skip registration toggle assertion without isolated database support");
        return;
    }
    let _ = run_mysql(
        Some(&server.database_name),
        "INSERT INTO site_settings (id, payload) VALUES (1, JSON_OBJECT('allow_user_register', TRUE)) ON DUPLICATE KEY UPDATE payload = JSON_SET(COALESCE(payload, JSON_OBJECT()), '$.allow_user_register', TRUE);",
    );
    let client = reqwest::Client::new();
    let admin_username = format!("evt_admin_toggle_{}", unique_suffix());
    let password = "Passw0rd_123";

    let register = client
        .post(format!("{}/v1/auth/register", server.base_url))
        .json(&serde_json::json!({
            "username": admin_username,
            "password": password,
        }))
        .send()
        .await
        .expect("register admin candidate");
    assert_eq!(register.status(), StatusCode::OK);

    let login = client
        .post(format!("{}/v1/auth/login", server.base_url))
        .json(&serde_json::json!({
            "username": admin_username,
            "password": password,
        }))
        .send()
        .await
        .expect("login admin candidate");
    assert_eq!(login.status(), StatusCode::OK);
    let login_body: Value = login.json().await.expect("login json");
    let token = login_body["data"]["token"]
        .as_str()
        .expect("login token")
        .to_string();

    let current_user = client
        .get(format!("{}/v1/users/me", server.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("current user response");
    assert_eq!(current_user.status(), StatusCode::OK);
    let current_user_body: Value = current_user.json().await.expect("current user json");
    let user_id = current_user_body["data"]["id"]
        .as_i64()
        .expect("current user id");
    server.register_test_user(user_id);
    if current_user_body["data"]["is_admin"] != true {
        server.promote_user_to_admin(user_id);
    }

    let save_settings = client
        .post(format!("{}/v1/admin/settings/save", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "items": [
                { "key": "web_profile.allow_user_register", "value": false }
            ]
        }))
        .send()
        .await
        .expect("disable registration response");
    assert_eq!(save_settings.status(), StatusCode::OK);

    let site_profile = client
        .get(format!("{}/v1/site/profile", server.base_url))
        .send()
        .await
        .expect("site profile response");
    assert_eq!(site_profile.status(), StatusCode::OK);
    let site_profile_body: Value = site_profile.json().await.expect("site profile json");
    assert_eq!(site_profile_body["data"]["allow_user_register"], false);

    let blocked_register = client
        .post(format!("{}/v1/auth/register", server.base_url))
        .json(&serde_json::json!({
            "username": format!("evt_blocked_{}", unique_suffix()),
            "password": password,
        }))
        .send()
        .await
        .expect("blocked register response");
    assert_eq!(blocked_register.status(), StatusCode::BAD_REQUEST);
    let blocked_register_body: Value = blocked_register
        .json()
        .await
        .expect("blocked register json");
    assert_eq!(blocked_register_body["code"], 400001);
    assert_eq!(
        blocked_register_body["msg"].as_str(),
        Some("user registration is disabled")
    );
}

#[tokio::test]
async fn local_http_e2e_admin_settings_disable_phone_bind_takes_effect_immediately() {
    let _guard = lock_e2e_guard();
    let mut server = LocalServer::start().await;
    if !server.isolated_database {
        eprintln!("skip phone bind toggle assertion without isolated database support");
        return;
    }
    let _ = run_mysql(
        Some(&server.database_name),
        "INSERT INTO site_settings (id, payload) VALUES (1, JSON_OBJECT('allow_phone_bind', TRUE)) ON DUPLICATE KEY UPDATE payload = JSON_SET(COALESCE(payload, JSON_OBJECT()), '$.allow_phone_bind', TRUE);",
    );
    let client = reqwest::Client::new();
    let admin_username = format!("evt_admin_phone_{}", unique_suffix());
    let password = "Passw0rd_123";

    let register = client
        .post(format!("{}/v1/auth/register", server.base_url))
        .json(&serde_json::json!({
            "username": admin_username,
            "password": password,
        }))
        .send()
        .await
        .expect("register phone-bind admin candidate");
    assert_eq!(register.status(), StatusCode::OK);

    let login = client
        .post(format!("{}/v1/auth/login", server.base_url))
        .json(&serde_json::json!({
            "username": admin_username,
            "password": password,
        }))
        .send()
        .await
        .expect("login phone-bind admin candidate");
    assert_eq!(login.status(), StatusCode::OK);
    let login_body: Value = login.json().await.expect("login json");
    let token = login_body["data"]["token"]
        .as_str()
        .expect("login token")
        .to_string();

    let current_user = client
        .get(format!("{}/v1/users/me", server.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("current user response");
    assert_eq!(current_user.status(), StatusCode::OK);
    let current_user_body: Value = current_user.json().await.expect("current user json");
    let user_id = current_user_body["data"]["id"]
        .as_i64()
        .expect("current user id");
    server.register_test_user(user_id);
    if current_user_body["data"]["is_admin"] != true {
        server.promote_user_to_admin(user_id);
    }

    let save_settings = client
        .post(format!("{}/v1/admin/settings/save", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "items": [
                { "key": "web_profile.allow_phone_bind", "value": false }
            ]
        }))
        .send()
        .await
        .expect("disable phone bind response");
    assert_eq!(save_settings.status(), StatusCode::OK);

    let site_profile = client
        .get(format!("{}/v1/site/profile", server.base_url))
        .send()
        .await
        .expect("site profile response");
    assert_eq!(site_profile.status(), StatusCode::OK);
    let site_profile_body: Value = site_profile.json().await.expect("site profile json");
    assert_eq!(site_profile_body["data"]["allow_phone_bind"], false);

    let blocked_phone_bind = client
        .post(format!("{}/v1/user/phone", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "phone": "13800138000",
            "captcha": "123456",
        }))
        .send()
        .await
        .expect("blocked phone bind response");
    assert_eq!(blocked_phone_bind.status(), StatusCode::BAD_REQUEST);
    let blocked_phone_bind_body: Value = blocked_phone_bind
        .json()
        .await
        .expect("blocked phone bind json");
    assert_eq!(blocked_phone_bind_body["code"], 400001);
    assert_eq!(
        blocked_phone_bind_body["msg"].as_str(),
        Some("phone binding is disabled")
    );
}

#[tokio::test]
async fn local_http_e2e_message_read_requires_the_receiver() {
    let _guard = lock_e2e_guard();
    let mut server = LocalServer::start().await;
    let client = reqwest::Client::new();

    let sender_username = format!("evt_sender_{}", unique_suffix());
    let receiver_username = format!("evt_receiver_{}", unique_suffix());
    let password = "Passw0rd_123";

    for username in [&sender_username, &receiver_username] {
        let register = client
            .post(format!("{}/v1/auth/register", server.base_url))
            .json(&serde_json::json!({
                "username": username,
                "password": password,
            }))
            .send()
            .await
            .expect("register message test user");
        assert_eq!(register.status(), StatusCode::OK);
    }

    let sender_login = client
        .post(format!("{}/v1/auth/login", server.base_url))
        .json(&serde_json::json!({
            "username": sender_username,
            "password": password,
        }))
        .send()
        .await
        .expect("login sender response");
    assert_eq!(sender_login.status(), StatusCode::OK);
    let sender_login_body: Value = sender_login.json().await.expect("sender login json");
    let sender_token = sender_login_body["data"]["token"]
        .as_str()
        .expect("sender token")
        .to_string();

    let receiver_login = client
        .post(format!("{}/v1/auth/login", server.base_url))
        .json(&serde_json::json!({
            "username": receiver_username,
            "password": password,
        }))
        .send()
        .await
        .expect("login receiver response");
    assert_eq!(receiver_login.status(), StatusCode::OK);
    let receiver_login_body: Value = receiver_login.json().await.expect("receiver login json");
    let receiver_token = receiver_login_body["data"]["token"]
        .as_str()
        .expect("receiver token")
        .to_string();

    for token in [&sender_token, &receiver_token] {
        let current_user = client
            .get(format!("{}/v1/users/me", server.base_url))
            .bearer_auth(token)
            .send()
            .await
            .expect("current user response");
        assert_eq!(current_user.status(), StatusCode::OK);
        let current_user_body: Value = current_user.json().await.expect("current user json");
        server.register_test_user(
            current_user_body["data"]["id"]
                .as_i64()
                .expect("current user id"),
        );
    }

    let send_message = client
        .post(format!("{}/v1/messages", server.base_url))
        .bearer_auth(&sender_token)
        .json(&serde_json::json!({
            "receiver_username": receiver_username,
            "content": "evt direct message permission check"
        }))
        .send()
        .await
        .expect("send message response");
    assert_eq!(send_message.status(), StatusCode::OK);
    let send_message_body: Value = send_message.json().await.expect("send message json");
    let message_id = send_message_body["data"]["id"]
        .as_i64()
        .expect("message id");

    let receiver_unread_before = client
        .get(format!("{}/v1/messages/unread-count", server.base_url))
        .bearer_auth(&receiver_token)
        .send()
        .await
        .expect("receiver unread before response");
    assert_eq!(receiver_unread_before.status(), StatusCode::OK);
    let receiver_unread_before_body: Value = receiver_unread_before
        .json()
        .await
        .expect("receiver unread before json");
    assert_eq!(receiver_unread_before_body["data"]["unread_count"], 1);

    let sender_mark_read = client
        .post(format!(
            "{}/v1/messages/{message_id}/read",
            server.base_url
        ))
        .bearer_auth(&sender_token)
        .send()
        .await
        .expect("sender mark read response");
    assert_eq!(sender_mark_read.status(), StatusCode::UNAUTHORIZED);

    let receiver_unread_after_sender_attempt = client
        .get(format!("{}/v1/messages/unread-count", server.base_url))
        .bearer_auth(&receiver_token)
        .send()
        .await
        .expect("receiver unread after sender attempt response");
    assert_eq!(receiver_unread_after_sender_attempt.status(), StatusCode::OK);
    let receiver_unread_after_sender_attempt_body: Value = receiver_unread_after_sender_attempt
        .json()
        .await
        .expect("receiver unread after sender attempt json");
    assert_eq!(
        receiver_unread_after_sender_attempt_body["data"]["unread_count"],
        1
    );

    let receiver_mark_read = client
        .post(format!(
            "{}/v1/messages/{message_id}/read",
            server.base_url
        ))
        .bearer_auth(&receiver_token)
        .send()
        .await
        .expect("receiver mark read response");
    assert_eq!(receiver_mark_read.status(), StatusCode::OK);

    let receiver_unread_after_read = client
        .get(format!("{}/v1/messages/unread-count", server.base_url))
        .bearer_auth(&receiver_token)
        .send()
        .await
        .expect("receiver unread after read response");
    assert_eq!(receiver_unread_after_read.status(), StatusCode::OK);
    let receiver_unread_after_read_body: Value = receiver_unread_after_read
        .json()
        .await
        .expect("receiver unread after read json");
    assert_eq!(receiver_unread_after_read_body["data"]["unread_count"], 0);
}

#[tokio::test]
async fn local_grpc_message_e2e_supports_whisper_unread_and_mark_read() {
    let _guard = lock_e2e_guard();
    let mut server = LocalServer::start().await;
    let client = reqwest::Client::new();

    let first_username = format!("grpc_sender_{}", unique_suffix());
    let first_password = "evt-password-1";
    let first_register = client
        .post(format!("{}/v1/auth/register", server.base_url))
        .json(&serde_json::json!({
            "username": first_username,
            "password": first_password,
            "nickname": "Grpc Sender"
        }))
        .send()
        .await
        .expect("register grpc sender");
    assert_eq!(first_register.status(), StatusCode::OK);
    let first_register_body: Value = first_register.json().await.expect("sender register json");
    let first_user_id = first_register_body["data"]["id"]
        .as_i64()
        .expect("sender id");

    let second_username = format!("grpc_receiver_{}", unique_suffix());
    let second_password = "evt-password-2";
    let second_register = client
        .post(format!("{}/v1/auth/register", server.base_url))
        .json(&serde_json::json!({
            "username": second_username,
            "password": second_password,
            "nickname": "Grpc Receiver"
        }))
        .send()
        .await
        .expect("register grpc receiver");
    assert_eq!(second_register.status(), StatusCode::OK);
    let second_register_body: Value = second_register
        .json()
        .await
        .expect("receiver register json");
    let second_user_id = second_register_body["data"]["id"]
        .as_i64()
        .expect("receiver id");
    server.test_user_ids.extend([first_user_id, second_user_id]);

    let first_login = client
        .post(format!("{}/v1/auth/login", server.base_url))
        .json(&serde_json::json!({
            "username": first_username,
            "password": first_password
        }))
        .send()
        .await
        .expect("login grpc sender");
    assert_eq!(first_login.status(), StatusCode::OK);
    let first_token = first_login
        .json::<Value>()
        .await
        .expect("sender login json")["data"]["token"]
        .as_str()
        .expect("sender token")
        .to_string();

    let second_login = client
        .post(format!("{}/v1/auth/login", server.base_url))
        .json(&serde_json::json!({
            "username": second_username,
            "password": second_password
        }))
        .send()
        .await
        .expect("login grpc receiver");
    assert_eq!(second_login.status(), StatusCode::OK);
    let second_token = second_login
        .json::<Value>()
        .await
        .expect("receiver login json")["data"]["token"]
        .as_str()
        .expect("receiver token")
        .to_string();

    let mut grpc_client = MessageServiceClient::connect(format!(
        "http://127.0.0.1:{}",
        server.grpc_port
    ))
        .await
        .expect("connect grpc message client");

    let unread_before = grpc_client
        .legacy_unread_count(UnreadCountRequest {
            bearer_token: second_token.clone(),
        })
        .await
        .expect("grpc unread before")
        .into_inner();
    assert_eq!(unread_before.status_code, 0);

    let whisper = grpc_client
        .send_legacy_whisper(SendLegacyWhisperRequest {
            bearer_token: first_token.clone(),
            user_id: second_user_id,
            content: "grpc hello from sender".into(),
        })
        .await
        .expect("grpc send whisper")
        .into_inner();
    assert_eq!(whisper.status_code, 0);

    let unread_after = grpc_client
        .legacy_unread_count(UnreadCountRequest {
            bearer_token: second_token.clone(),
        })
        .await
        .expect("grpc unread after")
        .into_inner();
    assert_eq!(unread_after.status_code, 0);
    assert_eq!(unread_after.unread_count, unread_before.unread_count + 1);

    let messages = grpc_client
        .list_legacy_messages(ListLegacyMessagesRequest {
            bearer_token: second_token.clone(),
            style: "whisper".into(),
            page: 1,
            page_size: 20,
        })
        .await
        .expect("grpc list whisper messages")
        .into_inner();
    assert_eq!(messages.status_code, 0);

    let whisper_message = messages
        .items
        .iter()
        .find(|item| {
            item.sender_user_id == first_user_id
                && item.receiver_user_id == second_user_id
                && item.content == "grpc hello from sender"
        })
        .expect("grpc whisper message");

    let mark_read = grpc_client
        .mark_read(MarkReadRequest {
            bearer_token: second_token,
            message_id: whisper_message.id,
        })
        .await
        .expect("grpc mark read")
        .into_inner();
    assert_eq!(mark_read.status_code, 0);

    let unread_after_read = grpc_client
        .legacy_unread_count(UnreadCountRequest {
            bearer_token: first_token,
        })
        .await
        .expect("grpc unread after read")
        .into_inner();
    assert_eq!(unread_after_read.status_code, 0);
    assert_eq!(unread_after_read.unread_count, 0);
}
