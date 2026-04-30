use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub app: AppSettings,
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub jwt: JwtSettings,
    pub storage: StorageSettings,
    pub web: WebSettings,
    pub site: SiteSettings,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AppSettings {
    pub name: String,
    pub env: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ServerSettings {
    pub http: HttpSettings,
    pub grpc: GrpcSettings,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HttpSettings {
    pub host: String,
    pub port: u16,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GrpcSettings {
    pub host: String,
    pub port: u16,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct JwtSettings {
    pub secret: String,
    pub issuer: String,
    pub expire_seconds: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StorageSettings {
    pub local_dir: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct WebSettings {
    pub dist_dir: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SiteSettings {
    pub allow_user_register: bool,
    pub allow_phone_bind: bool,
    pub use_friendship: bool,
    pub enable_trends_bar: bool,
    pub enable_wallet: bool,
    pub allow_tweet_attachment: bool,
    pub allow_tweet_attachment_price: bool,
    pub allow_tweet_video: bool,
    pub default_tweet_max_length: u32,
    pub tweet_web_ellipsis_size: u32,
    pub tweet_mobile_ellipsis_size: u32,
    pub default_tweet_visibility: String,
    pub default_msg_loop_interval: u32,
    pub copyright_top: String,
    pub copyright_left: String,
    pub copyright_left_link: String,
    pub copyright_right: String,
    pub copyright_right_link: String,
}

impl Settings {
    pub fn load() -> Result<Self, config::ConfigError> {
        let mut builder = config::Config::builder()
            .add_source(config::File::with_name("config/default").required(true))
            .add_source(config::Environment::with_prefix("PAOPAO_RS").separator("__"));

        if let Ok(path) = std::env::var("PAOPAO_RS_CONFIG") {
            builder = builder.add_source(config::File::with_name(&path).required(true));
        } else {
            builder = builder.add_source(config::File::with_name("config/local").required(false));
        }

        builder.build()?.try_deserialize()
    }

    pub fn http_addr(&self) -> String {
        format!("{}:{}", self.server.http.host, self.server.http.port)
    }

    pub fn grpc_addr(&self) -> String {
        format!("{}:{}", self.server.grpc.host, self.server.grpc.port)
    }
}
