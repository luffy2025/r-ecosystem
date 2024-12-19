use anyhow::Result;
use dashmap::DashMap;
use futures::stream::SplitStream;
use futures::{SinkExt, StreamExt};
use std::fmt::Display;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_util::codec::{Framed, LinesCodec};
use tracing::level_filters::LevelFilter;
use tracing::{info, warn};
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer as _;

#[derive(Debug)]
struct Peer {
    username: String,
    stream: SplitStream<Framed<TcpStream, LinesCodec>>,
}

impl Peer {
    fn new(username: String, stream: SplitStream<Framed<TcpStream, LinesCodec>>) -> Self {
        Self { username, stream }
    }
}

#[derive(Debug)]
enum Message {
    UserJoined(String),
    UserLeft(String),
    Chat { sender: String, content: String },
}

#[derive(Debug, Default)]
struct State {
    peers: DashMap<SocketAddr, mpsc::Sender<Arc<Message>>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_ansi(true).with_filter(LevelFilter::DEBUG);
    tracing_subscriber::registry().with(layer).init();

    // console_subscriber::init();

    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on {}", addr);

    let state = Arc::new(State::default());

    loop {
        let (stream, raddr) = listener.accept().await?;
        info!("Accepted connection from: {}", raddr);
        let state_clone = Arc::clone(&state);
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, raddr, state_clone).await {
                warn!("Failed to handle client {}: {}", raddr, e);
            }
        });
    }
}

async fn handle_client(stream: TcpStream, raddr: SocketAddr, state: Arc<State>) -> Result<()> {
    let mut stream = Framed::new(stream, LinesCodec::new());
    stream.send("Please enter your username:").await?;

    let username = match stream.next().await {
        Some(Ok(username)) => username,
        Some(Err(e)) => return Err(e.into()),
        None => return Ok(()),
    };

    let mut peer = state.add(username, raddr, stream).await;
    state
        .broadcast(raddr, Arc::new(Message::user_joined(&peer.username)))
        .await;

    while let Some(line) = peer.stream.next().await {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                warn!("Failed to read line: {}", e);
                break;
            }
        };

        if line.is_empty() {
            continue;
        }

        if line.starts_with('/') {
            match line.split_once(' ') {
                None => {
                    match line.as_str() {
                        "/quit" => {
                            state
                                .broadcast(raddr, Arc::new(Message::user_left(&peer.username)))
                                .await;
                            state.peers.remove(&raddr);
                            return Ok(());
                        }
                        _ => {
                            warn!("Unknown command: {}", line);
                        }
                    };
                }
                _ => warn!("Unknown command: {}", line),
            }
        } else {
            state
                .broadcast(raddr, Arc::new(Message::chat(&peer.username, line)))
                .await;
        }
    }

    info!("Client {} disconnected", raddr);
    Ok(())
}

const MAX_MESSAGE_SIZE: usize = 1024;

impl State {
    async fn add(
        &self,
        username: impl Into<String>,
        raddr: SocketAddr,
        stream: Framed<TcpStream, LinesCodec>,
    ) -> Peer {
        let (tx, mut rx) = mpsc::channel(MAX_MESSAGE_SIZE);
        self.peers.insert(raddr, tx);

        let (mut sender, receiver) = stream.split();

        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Err(e) = sender.send(msg.to_string()).await {
                    warn!("Failed to send message: {}", e);
                    break;
                }
            }
        });

        Peer::new(username.into(), receiver)
    }

    async fn broadcast(&self, raddr: SocketAddr, message: Arc<Message>) {
        for peer in self.peers.iter() {
            if peer.key() == &raddr {
                continue;
            }
            if let Err(e) = peer.value().send(Arc::clone(&message)).await {
                warn!("Failed to broadcast message: {}", e);
                self.peers.remove(peer.key());
            }
        }
    }
}

impl Message {
    fn user_joined(username: impl Into<String>) -> Self {
        Self::UserJoined(username.into())
    }

    fn user_left(username: impl Into<String>) -> Self {
        Self::UserLeft(username.into())
    }

    fn chat(sender: impl Into<String>, content: impl Into<String>) -> Self {
        Self::Chat {
            sender: sender.into(),
            content: content.into(),
        }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserJoined(username) => write!(f, "[{} joined the chat]", username),
            Self::UserLeft(username) => write!(f, "[{} left the chat]", username),
            Self::Chat { sender, content } => write!(f, "{}: {}", sender, content),
        }
    }
}
