use anyhow::Result;
use axum::extract::Request;
use axum::response::IntoResponse;
use axum::routing::get;
use once_cell::sync::Lazy;
use opentelemetry::trace::TracerProvider;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::{RandomIdGenerator, Tracer};
use opentelemetry_sdk::{runtime, Resource};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::{sleep, Instant};
use tracing::level_filters::LevelFilter;
use tracing::{debug, info, instrument, warn};
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

static RESOURCE: Lazy<Resource> =
    Lazy::new(|| Resource::new(vec![KeyValue::new("service.name", "axum-tracing")]));

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

    let tracer = init_tracer()?;
    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(console)
        .with(file)
        .with(opentelemetry)
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

#[instrument]
async fn index_handler() -> impl IntoResponse {
    "Hello, World!"
}

#[instrument(name = "my_long_task_handler", fields(http.method = req.method().as_str(), http.path = req.uri().path()))]
async fn long_task_handler(req: Request) -> impl IntoResponse {
    debug!("Starting long task");
    sleep(Duration::from_millis(20)).await;
    long_task().await;
    warn!(
        http.status_code = 200,
        http.response.body = "Done",
        "Long task done!"
    );
    "Done!"
}

#[instrument(name = "my_long_task")]
async fn long_task() {
    let start = Instant::now();
    sleep(Duration::from_millis(200)).await;

    task1().await;
    task2().await;

    let t3 = task3();
    let t4 = task4();
    tokio::join!(t3, t4);

    let elapsed = start.elapsed().as_millis();
    warn!(elapsed.Duration = elapsed, "Long task took {:?}ms", elapsed);
}

#[instrument(name = "task1")]
async fn task1() {
    let start = Instant::now();
    sleep(Duration::from_millis(20)).await;
    info!("Task 1 took {:?}", start.elapsed());
}

#[instrument(name = "task2")]
async fn task2() {
    let start = Instant::now();
    sleep(Duration::from_millis(40)).await;
    info!("Task 2 took {:?}", start.elapsed());
}

#[instrument(name = "task3")]
async fn task3() {
    let start = Instant::now();
    sleep(Duration::from_millis(60)).await;
    info!("Task 3 took {:?}", start.elapsed());
}

#[instrument(name = "task4")]
async fn task4() {
    let start = Instant::now();
    sleep(Duration::from_millis(80)).await;
    info!("Task 4 took {:?}", start.elapsed());
}

fn init_tracer() -> Result<Tracer> {
    let tracer = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(
            opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .with_endpoint("http://localhost:4317")
                .build()?,
            runtime::Tokio,
        )
        .with_id_generator(RandomIdGenerator::default())
        .with_max_events_per_span(32)
        .with_max_attributes_per_span(64)
        .with_resource(RESOURCE.clone())
        .build()
        .tracer("example-tracer");

    Ok(tracer)
}
