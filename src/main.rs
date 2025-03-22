use std::io::Error;
use tokio::io::{AsyncReadExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::spawn;

async fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0u8; 1024];
    loop {
        let bytes_read = match stream.read(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };

        let message = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("Received: {}", &message[0..message.len()-1]);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on 127.0.0.1:8080");

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                spawn(async move { handle_client(stream).await });
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}
