use anyhow::Context;
use paopao_config::Settings;
use paopao_grpc_api::authenticate_service;
use paopao_http_api::{HttpState, router};
use paopao_infra::AppContext;
use tokio::net::TcpListener;
use tonic::transport::Server;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let settings = Settings::load().context("load rust backend settings")?;
    let app = AppContext::bootstrap(settings.clone()).await?;

    let http_addr = settings.http_addr();
    let grpc_addr = settings.grpc_addr();

    let http_state = HttpState::new(app.clone());
    let http_listener = TcpListener::bind(&http_addr)
        .await
        .with_context(|| format!("bind http listener on {http_addr}"))?;

    let http_server = async move {
        info!("http server listening on {}", http_addr);
        axum::serve(http_listener, router(http_state))
            .await
            .context("run http server")
    };

    let grpc_server = async move {
        info!("grpc server listening on {}", grpc_addr);
        Server::builder()
            .add_service(authenticate_service(app))
            .serve(grpc_addr.parse()?)
            .await
            .context("run grpc server")
    };

    tokio::select! {
        result = http_server => result,
        result = grpc_server => result,
        _ = tokio::signal::ctrl_c() => {
            info!("shutdown signal received");
            Ok(())
        }
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
