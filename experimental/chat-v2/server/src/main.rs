use dashmap::DashMap;
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::broadcast,
};

type Tx = broadcast::Sender<String>;
type ChatRooms = Arc<DashMap<String, Tx>>;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Server running on 127.0.0.1:8080");

    let rooms: ChatRooms = Arc::new(DashMap::new());

    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        let rooms = rooms.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, addr, rooms).await {
                println!("Error handling client {}: {:?}", addr, e);
            }
        });
    }
}

async fn handle_client(
    stream: TcpStream,
    addr: SocketAddr,
    rooms: ChatRooms,
) -> anyhow::Result<()> {
    let (reader, writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut writer = writer;

    let mut buffer = String::new();

    // Ask for room name
    writer.write_all(b"Enter room name: ").await?;
    reader.read_line(&mut buffer).await?;
    let room = buffer.trim().to_string();
    buffer.clear();

    let tx = rooms
        .entry(room.clone())
        .or_insert_with(|| broadcast::channel(100).0)
        .clone();

    let mut rx = tx.subscribe();

    writer
        .write_all(format!("Joined room '{}'. Start chatting!\n", room).as_bytes())
        .await?;

    loop {
        tokio::select! {
            // Handle input from the client
            result = reader.read_line(&mut buffer) => {
                match result {
                    Ok(0) => break, // client disconnected
                    Ok(_) => {
                        let msg = format!("[{}] {}", addr, buffer.trim());
                        let _ = tx.send(msg);
                        buffer.clear();
                    },
                    Err(e) => {
                        eprintln!("Read error from {}: {:?}", addr, e);
                        break;
                    }
                }
            }

            // Handle messages from the chat room
            result = rx.recv() => {
                match result {
                    Ok(msg) => {
                        if let Err(e) = writer.write_all(format!("{}\n", msg).as_bytes()).await {
                            eprintln!("Write error to {}: {:?}", addr, e);
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        let _ = writer.write_all(format!("⚠️ Skipped {} messages\n", skipped).as_bytes()).await;
                    }
                    Err(_) => break, // Sender dropped
                }
            }
        }
    }

    println!("Client {} disconnected from room '{}'", addr, room);
    Ok(())
}
