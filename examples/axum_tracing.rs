use anyhow::Result;
use axum::response::IntoResponse;
use axum::routing::get;
use tokio::net::TcpListener;
use tokio::time::{sleep, Instant};
use tracing::level_filters::LevelFilter;
use tracing::{debug, info, warn};
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

#[tokio::main]
async fn main() -> Result<()> {
    let file_appender = tracing_appender::rolling::daily("tmp/logs", "ecosystem.log");

    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let console = fmt::Layer::new()
        .with_span_events(FmtSpan::CLOSE)
        .with_ansi(true)
        .pretty()
        .with_filter(LevelFilter::DEBUG);

    let file = fmt::Layer::new()
        .with_span_events(FmtSpan::CLOSE)
        .with_ansi(false)
        .with_writer(non_blocking)
        .with_filter(LevelFilter::INFO);

    tracing_subscriber::registry()
        .with(console)
        .with(file)
        .init();

    let addr = "0.0.0.0:8080";
    let app = axum::Router::new()
        .route("/", get(index_handler))
        .route("/long", get(long_task_handler));
    info!("Listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

#[tracing::instrument]
async fn index_handler() -> impl IntoResponse {
    "Hello, World!"
}

#[tracing::instrument]
async fn long_task_handler() -> impl IntoResponse {
    debug!("Starting long task");
    long_task().await;
    warn!("Long task done!");
    "Done!"
}

#[tracing::instrument]
async fn long_task() {
    let start = Instant::now();
    sleep(tokio::time::Duration::from_millis(200)).await;
    warn!("Long task took {:?}", start.elapsed());
}
