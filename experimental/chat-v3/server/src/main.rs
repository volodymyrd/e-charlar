use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::mpsc,
};
use dashmap::DashMap;
use std::{collections::{HashMap, HashSet}, sync::Arc, sync::atomic::{AtomicUsize, Ordering}};

type ClientId = usize;
type Clients = Arc<DashMap<ClientId, Client>>;
type Rooms = Arc<DashMap<String, HashSet<ClientId>>>;

static NEXT_CLIENT_ID: AtomicUsize = AtomicUsize::new(1);

struct Client {
    id: ClientId,
    sender: mpsc::UnboundedSender<String>,
    rooms: HashSet<String>,
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Chat server running on 127.0.0.1:8080");

    let clients: Clients = Arc::new(DashMap::new());
    let rooms: Rooms = Arc::new(DashMap::new());

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let clients = clients.clone();
        let rooms = rooms.clone();

        tokio::spawn(async move {
            handle_client(stream, clients, rooms).await;
        });
    }
}

async fn handle_client(stream: TcpStream, clients: Clients, rooms: Rooms) {
    let id = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);
    let (reader, writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut writer = writer;
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let mut buffer = String::new();

    // Ask for room list
    writer.write_all(b"Enter rooms (comma-separated): ").await.ok();
    reader.read_line(&mut buffer).await.ok();
    let room_list = buffer.trim().split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>();
    buffer.clear();

    // Register client and subscriptions
    let mut client_rooms = HashSet::new();
    for room in &room_list {
        rooms.entry(room.clone()).or_default().insert(id);
        client_rooms.insert(room.clone());
    }

    clients.insert(id, Client { id, sender: tx.clone(), rooms: client_rooms });

    writer.write_all(b"Welcome! Use /send room_name message\n").await.ok();

    loop {
        tokio::select! {
            result = reader.read_line(&mut buffer) => {
                match result {
                    Ok(0) => break, // client disconnected
                    Ok(_) => {
                        let input = buffer.trim().to_string();
                        if input.starts_with("/send") {
                            let parts: Vec<_> = input.splitn(3, ' ').collect();
                            if parts.len() < 3 {
                                writer.write_all(b"Usage: /send room_name message\n").await.ok();
                            } else {
                                let room = parts[1];
                                let msg = parts[2];
                                if let Some(subscribers) = rooms.get(room) {
                                    for client_id in subscribers.iter() {
                                        if let Some(client) = clients.get(client_id) {
                                            let _ = client.sender.send(format!("[{}] {}", room, msg));
                                        }
                                    }
                                }
                            }
                        } else {
                            writer.write_all(b"Unknown command\n").await.ok();
                        }
                        buffer.clear();
                    }
                    Err(e) => {
                        eprintln!("Read error from client {}: {:?}", id, e);
                        break;
                    }
                }
            }

            Some(msg) = rx.recv() => {
                if writer.write_all(format!("{}\n", msg).as_bytes()).await.is_err() {
                    break; // write failed
                }
            }
        }
    }

    // Cleanup
    for room in &room_list {
        if let Some(mut subs) = rooms.get_mut(room) {
            subs.remove(&id);
        }
    }
    clients.remove(&id);
    println!("Client {} disconnected", id);
}
