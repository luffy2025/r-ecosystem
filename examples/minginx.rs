use anyhow::Result;
use std::sync::Arc;
use tokio::io;
use tokio::net::{TcpListener, TcpStream};
use tracing::level_filters::LevelFilter;
use tracing::{info, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, Layer};

struct Config {
    upstream_addr: String,
    listener_addr: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = fmt::Layer::new()
        .with_ansi(true)
        .with_filter(LevelFilter::DEBUG);
    tracing_subscriber::registry().with(layer).init();

    let config = Arc::new(resolve_config());

    let listener = TcpListener::bind(&config.listener_addr).await?;
    info!("listening on {}", config.listener_addr);

    loop {
        let (client, addr) = listener.accept().await?;
        info!("accepted connection from {}", addr);
        let config_clone = Arc::clone(&config);
        tokio::spawn(async move {
            let upstream = TcpStream::connect(&config_clone.upstream_addr).await?;
            proxy(client, upstream).await?;
            Ok::<(), anyhow::Error>(())
        });
    }
}

async fn proxy(mut client: TcpStream, mut upstream: TcpStream) -> Result<()> {
    let (mut client_read, mut client_write) = client.split();
    let (mut upstream_read, mut upstream_write) = upstream.split();

    let client_to_upstream = io::copy(&mut client_read, &mut upstream_write);
    let upstream_to_client = io::copy(&mut upstream_read, &mut client_write);

    match tokio::try_join!(client_to_upstream, upstream_to_client) {
        Ok((client_bytes, upstream_bytes)) => {
            info!(
                "proxied {} bytes from client to upstream and {} bytes from upstream to client",
                client_bytes, upstream_bytes
            );
        }
        Err(e) => {
            warn!("proxy failed: {}", e);
        }
    }

    Ok(())
}

fn resolve_config() -> Config {
    Config {
        upstream_addr: "0.0.0.0:8080".to_string(),
        listener_addr: "0.0.0.0:8081".to_string(),
    }
}
