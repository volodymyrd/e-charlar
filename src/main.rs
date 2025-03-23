use std::collections::HashMap;
use std::io::Error;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, broadcast};

async fn handle_client(stream: TcpStream, tx: broadcast::Sender<(i64, String)>, users: UserMap) {
    let (mut reader, mut writer) = stream.into_split();

    let client_id = rand::random::<i64>();

    let mut buffer = [0u8; 1024];

    // Request username
    writer.write_all(b"Enter your username: ").await.unwrap();
    let bytes_read = reader.read(&mut buffer).await.unwrap();
    let username = String::from_utf8_lossy(&buffer[..bytes_read])
        .trim()
        .to_string();
    users.lock().await.insert(client_id, username.clone());
    println!("{} has joined the chat.", username);

    let mut rx = tx.subscribe();

    let users_clone = Arc::clone(&users);
    tokio::spawn(async move {
        while let Ok((id, msg)) = rx.recv().await {
            if id == client_id {
                continue;
            }

            let sender_name = users_clone
                .lock()
                .await
                .get(&id)
                .unwrap_or(&"Unknown".to_string())
                .clone();
            let formatted_msg = format!("{}: {}", sender_name, msg);
            if (writer.write_all(formatted_msg.as_bytes()).await).is_err() {
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
        println!("Received: {}", message.trim());
        if tx.send((client_id, message.into_owned())).is_err() {
            break;
        }
    }

    users.lock().await.remove(&client_id);
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
                tokio::spawn(async move { handle_client(stream, tx, users).await });
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}
