use anyhow::Context;
use evt_config::Settings;
use evt_grpc_api::authenticate_service;
use evt_http_api::{HttpState, router};
use evt_infra::AppContext;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use tokio::net::TcpListener;
use tonic::transport::Server;
use tracing::info;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = Settings::load().context("load rust backend settings")?;
    let telemetry = init_tracing(&settings)?;
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
            if let Some(provider) = telemetry {
                provider.shutdown()?;
            }
            Ok(())
        }
    }
}

fn init_tracing(
    settings: &Settings,
) -> anyhow::Result<Option<opentelemetry_sdk::trace::SdkTracerProvider>> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let fmt_layer = tracing_subscriber::fmt::layer();

    if !settings.telemetry.enabled {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .init();
        return Ok(None);
    }

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(settings.telemetry.otlp_endpoint.clone())
        .build()?;

    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .build();

    let tracer = provider.tracer(settings.telemetry.service_name.clone());
    let telemetry_layer = OpenTelemetryLayer::new(tracer);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .with(telemetry_layer)
        .init();

    Ok(Some(provider))
}
