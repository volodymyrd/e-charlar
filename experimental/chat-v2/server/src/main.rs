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
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut buffer = String::new();

    // Ask for a room name
    writer.write_all(b"Enter room name: ").await?;
    reader.read_line(&mut buffer).await?;
    let room = buffer.trim().to_string();
    buffer.clear();

    // Get or create the broadcast channel for the room
    let tx = rooms
        .entry(room.clone())
        .or_insert_with(|| broadcast::channel(100).0)
        .clone();

    let mut rx = tx.subscribe();

    // Welcome message
    writer
        .write_all(format!("Joined room '{}'. You can start chatting!\n", room).as_bytes())
        .await?;

    // Spawn a task to forward incoming messages to this client
    let writer_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Err(e) = writer.write_all(format!("{}\n", msg).as_bytes()).await {
                eprintln!("Error writing to client {}: {:?}", addr, e);
                break;
            }
        }
    });

    // Read messages from the client and broadcast them
    while reader.read_line(&mut buffer).await? != 0 {
        let msg = format!("[{}] {}", addr, buffer.trim());
        let _ = tx.send(msg);
        buffer.clear();
    }

    println!("Client {} disconnected", addr);
    Ok(())
}
