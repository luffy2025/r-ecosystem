use anyhow::Result;
use std::thread;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel(16);
    let handler = worker(rx);

    tokio::spawn(async move {
        let mut i = 0;
        loop {
            i += 1;
            println!("sending {}", i);
            tx.send(format!("hello {}", i).to_string())
                .await
                .expect("channel closed");
        }
    });

    handler
        .join()
        .map_err(|e| anyhow::anyhow!("worker thread panicked {:?}", e))?;
    Ok(())
}

fn worker(mut rx: mpsc::Receiver<String>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let (sender, receiver) = std::sync::mpsc::channel();
        while let Some(s) = rx.blocking_recv() {
            let sender_clone = sender.clone();
            thread::spawn(move || {
                let result = expensive_blocking_task(s);
                sender_clone.send(result).unwrap();
            });
            let result = receiver.recv().unwrap();
            println!("got result: {}", result);
        }
    })
}

fn expensive_blocking_task(s: String) -> String {
    println!("computing: {}", s);
    thread::sleep(std::time::Duration::from_secs(1));
    blake3::hash(s.as_bytes()).to_string()
}
