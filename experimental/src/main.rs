use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};

async fn handle_client(
    stream: TcpStream,
    tx: broadcast::Sender<(i64, String)>,
) -> anyhow::Result<()> {
    let (reader, writer) = stream.into_split();

    let client_id = rand::random::<i64>();

    let mut stream = FramedRead::new(reader, LinesCodec::new());
    let mut sink = FramedWrite::new(writer, LinesCodec::new());

    let mut rx = tx.subscribe();

    println!("Start new connection!!!");

    loop {
        tokio::select! {
            user_message = stream.next() => {
                match user_message {
                    None => break,
                    Some(user_message) => {
                        let user_message  = user_message?;
                        println!("Received: {}", user_message.trim());
                        tx.send((client_id, user_message))?
                    },
                };
            },
            recv = rx.recv() => {
                let (id, peer_message) = recv?;
                if client_id != id {
                    sink.send(peer_message).await?;
                }
            }
        }
    }

    Ok(())
}

const MAX_CLIENTS: usize = 100;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on 127.0.0.1:8080");

    let (tx, _) = broadcast::channel(MAX_CLIENTS);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let tx = tx.clone();
                tokio::spawn(handle_client(stream, tx));
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}
