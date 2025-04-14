use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::{Mutex, OnceCell};

static CONNECTION: OnceCell<Mutex<TcpStream>> = OnceCell::const_new();

#[tauri::command]
async fn connect_to_server() -> Result<(), String> {
    println!("Try connecting to server...");
    if is_connected().await? {
        println!("Connection has already established...");
        return Ok(());
    }
    let stream = TcpStream::connect("127.0.0.1:8080")
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;
    CONNECTION
        .set(Mutex::new(stream))
        .map_err(|_| "Connection already set".to_string())?;
    Ok(())
}

#[tauri::command]
async fn send_message(mut message: String) -> Result<String, String> {
    message.push('\n');
    message.push('\r');
    let mutex = CONNECTION
        .get()
        .ok_or("Not connected to server".to_string())?;
    let mut stream = mutex.lock().await;

    stream
        .write_all(message.as_bytes())
        .await
        .map_err(|e| e.to_string())?;

    Ok(("OK").to_string())
}

async fn is_connected() -> Result<bool, String> {
    Ok(CONNECTION.get().is_some())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![connect_to_server, send_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
