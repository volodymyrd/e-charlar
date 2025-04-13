use crate::cli::Cli;
use crate::logging::set_up_logging;
use clap::Parser;
use tokio::net::TcpListener;
use tracing::{debug, info};

mod cli;
mod logging;

/// Default port that a chat server listens on.
///
/// Used if no port is specified.
const DEFAULT_PORT: u16 = 6379;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    set_up_logging()?;

    info!("Starting...");

    let cli = Cli::parse();
    let port = cli.port.unwrap_or(DEFAULT_PORT);

    debug!("Binding a TCP listener on port {port}...");

    let listener = TcpListener::bind(&format!("127.0.0.1:{}", port)).await?;

    Ok(())
}
