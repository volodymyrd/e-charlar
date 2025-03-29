use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::io::Error;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, broadcast};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};

async fn handle_client(stream: TcpStream, tx: broadcast::Sender<(i64, String)>, users: UserMap) {
    let (reader, writer) = stream.into_split();

    let client_id = rand::random::<i64>();

    let mut stream = FramedRead::new(reader, LinesCodec::new());
    let mut sink = FramedWrite::new(writer, LinesCodec::new());

    // Request username
    if (sink.send("Enter your username: ").await).is_err() {
        return;
    }
    if let Some(Ok(username)) = stream.next().await {
        users.lock().await.insert(client_id, username.clone());
        println!("{} has joined the chat.", username);
        if (sink.send(format!("Hello {username}! ❤️")).await).is_err() {
            return;
        }
    }

    let mut rx = tx.subscribe();

    let users_clone = Arc::clone(&users);

    loop {
        tokio::select! {
            user_message = stream.next() => {
                match user_message {
                    None => break,
                    Some(user_message) => {
                        if user_message.is_err() {
                            break;
                        }
                        let user_message  = user_message.unwrap();
                        println!("Received: {}", user_message.trim());
                        if tx.send((client_id, user_message)).is_err() {
                            break;
                        }
                    },
                };
            },
            recv = rx.recv() => {
                if recv.is_err() {
                    break;
                }
                let (id, peer_message) = recv.unwrap();
                if client_id != id {
                    let sender_name = users_clone.lock().await.get(&id)
                    .unwrap_or(&"Unknown".to_string()).clone();
                    let formatted_msg = format!("{}: {}", sender_name, peer_message);
                    if (sink.send(formatted_msg).await).is_err() {
                        break;
                    }
                }
            }
        }
    }

    let username = users.lock().await.remove(&client_id).unwrap();
    println!("{} has left the chat.", username);
}

type UserMap = Arc<Mutex<HashMap<i64, String>>>;
const MAX_CLIENTS: usize = 100;

#[tokio::main]
async fn main() -> Result<(), Box<Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on 127.0.0.1:8080");

    let users: UserMap = Arc::new(Mutex::new(HashMap::new()));

    let (tx, _) = broadcast::channel(MAX_CLIENTS);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let tx = tx.clone();
                let users = Arc::clone(&users);
                tokio::spawn(handle_client(stream, tx, users));
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}
