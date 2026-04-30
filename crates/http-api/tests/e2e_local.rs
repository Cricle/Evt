use std::{
    fs::File,
    net::TcpListener as StdTcpListener,
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use evt_config::{
    AppSettings, DatabaseSettings, GrpcSettings, HttpSettings, JwtSettings, ServerSettings,
    Settings, SiteSettings, StorageSettings, WebSettings,
};
use reqwest::StatusCode;
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

fn run_mysql(database: Option<&str>, sql: &str) -> bool {
    Command::new("mysql")
        .args(mysql_args(database, sql))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
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
    child: Child,
    database_name: String,
    isolated_database: bool,
    shared_cleanup_database: String,
    test_user_id: Option<i64>,
}

impl LocalServer {
    async fn start() -> Self {
        let suffix = unique_suffix();
        let http_port = reserve_port();
        let grpc_port = reserve_port();
        let database_name = format!("evt_e2e_{suffix}");
        let shared_cleanup_database = mysql_env("MYSQL_CLEAN_DATABASE", "evt");
        let isolated_database = run_mysql(
            None,
            &format!(
                "DROP DATABASE IF EXISTS `{database_name}`; CREATE DATABASE `{database_name}` CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci;"
            ),
        );
        let active_database = if isolated_database {
            database_name.clone()
        } else {
            shared_cleanup_database.clone()
        };

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
            web: WebSettings {
                dist_dir: web_dist_dir.display().to_string(),
            },
            site: SiteSettings {
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
            "[app]\nname = \"{}\"\nenv = \"{}\"\n\n[server.http]\nhost = \"{}\"\nport = {}\n\n[server.grpc]\nhost = \"{}\"\nport = {}\n\n[database]\nurl = \"{}\"\nmax_connections = {}\n\n[jwt]\nsecret = \"{}\"\nissuer = \"{}\"\nexpire_seconds = {}\n\n[storage]\nlocal_dir = \"{}\"\n\n[web]\ndist_dir = \"{}\"\n\n[site]\nallow_user_register = {}\nallow_phone_bind = {}\nuse_friendship = {}\nenable_trends_bar = {}\nenable_wallet = {}\nallow_tweet_attachment = {}\nallow_tweet_attachment_price = {}\nallow_tweet_video = {}\ndefault_tweet_max_length = {}\ntweet_web_ellipsis_size = {}\ntweet_mobile_ellipsis_size = {}\ndefault_tweet_visibility = \"{}\"\ndefault_msg_loop_interval = {}\ncopyright_top = \"{}\"\ncopyright_left = \"{}\"\ncopyright_left_link = \"{}\"\ncopyright_right = \"{}\"\ncopyright_right_link = \"{}\"\n",
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
            settings.web.dist_dir,
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
        let stderr = stdout
            .try_clone()
            .expect("clone server log handle");

        let binary = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/debug/evt");
        let child = Command::new(binary)
            .current_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
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
                        child,
                        database_name: active_database,
                        isolated_database,
                        shared_cleanup_database,
                        test_user_id: None,
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

        if let Some(user_id) = self.test_user_id {
            let cleanup_db = if self.isolated_database {
                self.database_name.as_str()
            } else {
                self.shared_cleanup_database.as_str()
            };
            let _ = run_mysql(Some(cleanup_db), &format!("DELETE FROM users WHERE id = {user_id};"));
        }

        if self.isolated_database {
            let _ = run_mysql(None, &format!("DROP DATABASE IF EXISTS `{}`;", self.database_name));
        }
    }
}

#[tokio::test]
async fn local_http_e2e_covers_web_and_legacy_post_comment_flow() {
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
    assert!(asset.text().await.expect("asset text").contains("evt-e2e-asset"));

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
    server.test_user_id = user_info_body["data"]["id"].as_i64();

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

    let create_post = client
        .post(format!("{}/v1/post", server.base_url))
        .bearer_auth(&token)
        .json(&serde_json::json!({
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
    let create_comment_body: Value = create_comment
        .json()
        .await
        .expect("create comment json");
    assert_eq!(create_comment_body["code"], 0);
    assert_eq!(
        create_comment_body["data"]["contents"][0]["content"].as_str(),
        Some("evt e2e comment")
    );

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
        let list_comments_body: Value = list_comments
            .json()
            .await
            .expect("list comments json");
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
}
