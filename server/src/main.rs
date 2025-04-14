use crate::cli::Cli;
use crate::logging::set_up_logging;
use clap::Parser;
use tokio::net::TcpListener;
use tokio::signal;
use tracing::{debug, info};

mod cli;
mod logging;
mod server;

/// Default port that a chat server listens on.
///
/// Used if no port is specified.
const DEFAULT_PORT: u16 = 6379;

/// Maximum number of concurrent connections the chat server will accept.
///
/// When this limit is reached, the server will stop accepting connections until
/// an active connection terminates.
const MAX_CONNECTIONS: usize = 250;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    set_up_logging()?;

    info!("Starting...");

    let cli = Cli::parse();
    let port = cli.port.unwrap_or(DEFAULT_PORT);
    let max_connections = cli.max_connections.unwrap_or(MAX_CONNECTIONS);

    debug!("Binding a TCP listener on port {port}...");

    let listener = TcpListener::bind(&format!("127.0.0.1:{}", port)).await?;

    server::run(listener, max_connections, signal::ctrl_c()).await;

    Ok(())
}
