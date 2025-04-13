use clap::Parser;
use crate::cli::Cli;

mod cli;

/// Default port that a chat server listens on.
///
/// Used if no port is specified.
const DEFAULT_PORT: u16 = 6379;

fn main() {
    let cli = Cli::parse();
    let port = cli.port.unwrap_or(DEFAULT_PORT);
}
