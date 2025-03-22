use std::io::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;

async fn handle_client(stream: TcpStream, tx: broadcast::Sender<String>) {
    let (mut reader, mut writer) = stream.into_split();

    let mut buffer = [0u8; 1024];

    let mut rx = tx.subscribe();

    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if (writer.write_all(msg.as_bytes()).await).is_err() {
                break;
            }
        }
    });

    loop {
        let bytes_read = match reader.read(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };

        let message = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("Received: {}", message);
        if tx.send(message.into_owned()).is_err() {
            break;
        }
    }
}

const MAX_CLIENTS: usize = 100;

#[tokio::main]
async fn main() -> Result<(), Box<Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on 127.0.0.1:8080");

    let (tx, _) = broadcast::channel(MAX_CLIENTS);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let tx = tx.clone();
                tokio::spawn(async move { handle_client(stream, tx).await });
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}
