use anyhow::Result;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{get, patch};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, Layer};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    name: String,
    email: Option<String>,
    age: u32,
    dob: chrono::DateTime<chrono::Utc>,
    skills: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserUpdate {
    name: Option<String>,
    email: Option<String>,
    skills: Option<Vec<String>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let console = fmt::Layer::new()
        .with_span_events(FmtSpan::CLOSE)
        .with_ansi(true)
        .pretty()
        .with_filter(LevelFilter::DEBUG);
    tracing_subscriber::registry().with(console).init();

    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on {}", addr);

    let user = User {
        name: "Cherry".to_string(),
        email: Some("cherry@xyz.com".to_string()),
        age: 22,
        dob: chrono::Utc::now(),
        skills: vec!["Rust".to_string(), "Python".to_string()],
    };

    let user = Arc::new(Mutex::new(user));

    let router = axum::Router::new()
        .route("/", get(user_handler))
        .route("/state", get(user_state_handler))
        .route("/update", patch(update_state_handler))
        .with_state(user);

    axum::serve(listener, router.into_make_service()).await?;

    Ok(())
}

async fn user_handler() -> impl IntoResponse {
    let user = User {
        name: "Bob".to_string(),
        email: Some("bob@xyz.com".to_string()),
        age: 26,
        dob: chrono::Utc::now(),
        skills: vec!["Rust".to_string(), "Python".to_string()],
    };

    let resp: Json<User> = user.into();
    resp
}

async fn user_state_handler(State(user): State<Arc<Mutex<User>>>) -> impl IntoResponse {
    let user: User = user.lock().unwrap().clone();
    let resp: Json<User> = user.into();
    resp
}

async fn update_state_handler(
    State(user): State<Arc<Mutex<User>>>,
    Json(update): Json<UserUpdate>,
) -> impl IntoResponse {
    let mut user = user.lock().unwrap();
    if let Some(name) = update.name {
        user.name = name;
    }
    if let Some(email) = update.email {
        user.email = Some(email);
    }
    if let Some(skills) = update.skills {
        user.skills = skills;
    }

    let resp: Json<User> = user.clone().into();
    resp
}
