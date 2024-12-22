use anyhow::Result;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use http::header::LOCATION;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::Error::{Database, RowNotFound};
use sqlx::{FromRow, PgPool};
use std::fmt::Debug;
use std::sync::Arc;
use thiserror::Error;
use tokio::net::TcpListener;
use tracing::{info, warn};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer as _;

const ADDR: &str = "http://127.0.0.1:4869/";

#[derive(Debug, FromRow)]
struct UrlRecord {
    #[sqlx(default)]
    id: String,
    #[sqlx(default)]
    url: String,
}

#[derive(Debug, Deserialize)]
struct ShortenReq {
    url: String,
}

#[derive(Debug, Serialize)]
struct ShortenRes {
    url: String,
}

struct AppState {
    db: PgPool,
}

#[derive(Error, Debug)]
enum AppError {
    #[error("db error")]
    SqlXFailed,
    #[error("failed to insert url")]
    DBFailed,
    #[error("id exists")]
    IdExists,
    #[error("id not found")]
    IdNotFound,
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_ansi(true).with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let addr = "0.0.0.0:4869";
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on {}", addr);

    let state = AppState::try_new("postgres://localhost/shortener").await?;
    let state = Arc::new(state);

    let router = Router::new()
        .route("/", post(shorten))
        .route("/:id", get(redirect))
        .with_state(state);

    axum::serve(listener, router.into_make_service()).await?;

    Ok(())
}

async fn shorten(
    State(state): State<Arc<AppState>>,
    Json(data): Json<ShortenReq>,
) -> Result<impl IntoResponse, AppError> {
    let record = state.insert_url(data.url).await?;

    let body = Json(ShortenRes {
        url: format!("{}{}", ADDR, record.id),
    });

    Ok((StatusCode::CREATED, body))
}

async fn redirect(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let url = state.get_url(id).await?;
    let mut headers = http::header::HeaderMap::new();
    headers.insert(LOCATION, url.parse().unwrap());

    Ok((StatusCode::PERMANENT_REDIRECT, headers))
}

impl AppState {
    async fn try_new(db_url: &str) -> Result<Self> {
        let pool = PgPool::connect(db_url).await?;

        sqlx::query(
            r#"
                CREATE TABLE IF NOT EXISTS urls (
                    id CHAR(6) PRIMARY KEY,
                    url TEXT NOT NULL UNIQUE
                )
                "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { db: pool })
    }

    async fn get_url(&self, id: impl Into<String>) -> Result<String, AppError> {
        let record: UrlRecord = sqlx::query_as("SELECT url FROM urls WHERE id = $1")
            .bind(id.into())
            .fetch_one(&self.db)
            .await
            .map_err(|e| match e {
                RowNotFound => AppError::IdNotFound,
                _ => {
                    warn!("db select error: {:?}", e);
                    AppError::DBFailed
                }
            })?;

        Ok(record.url)
    }

    async fn insert_url(&self, url: impl Into<String>) -> Result<UrlRecord, AppError> {
        let url = url.into();
        for _ in 0..20 {
            let id = nanoid::nanoid!(6);
            match self.insert_url_0(id, &url).await {
                Ok(record) => return Ok(record),
                Err(e) => match e {
                    AppError::IdExists => continue,
                    _ => return Err(e),
                },
            }
        }
        Err(AppError::DBFailed)
    }

    async fn insert_url_0(
        &self,
        id: impl Into<String>,
        url: impl Into<String>,
    ) -> Result<UrlRecord, AppError> {
        let record: UrlRecord = sqlx::query_as(
            r#"INSERT INTO urls (id, url) VALUES ($1, $2)
                ON CONFLICT(url) DO UPDATE SET url=EXCLUDED.url
                RETURNING id"#,
        )
        .bind(id.into())
        .bind(url.into())
        .fetch_one(&self.db)
        .await
        .map_err(|e| match e {
            Database(e) => {
                if e.is_unique_violation() {
                    AppError::IdExists
                } else {
                    warn!("db insert error: {:?}", e);
                    AppError::DBFailed
                }
            }
            _ => {
                warn!("sqlx error: {:?}", e);
                AppError::SqlXFailed
            }
        })?;

        Ok(record)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::SqlXFailed => {
                (StatusCode::BAD_REQUEST, "[1] Please try later").into_response()
            }
            AppError::DBFailed => {
                (StatusCode::UNPROCESSABLE_ENTITY, "[2] Please try later").into_response()
            }
            AppError::IdExists => {
                (StatusCode::PAYLOAD_TOO_LARGE, "[3] Please try later").into_response()
            }
            AppError::IdNotFound => (StatusCode::NOT_FOUND, "[4] URL not found").into_response(),
        }
    }
}
