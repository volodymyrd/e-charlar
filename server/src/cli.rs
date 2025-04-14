use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "e-charlar-server", version, about = "An Encrypted Chat Server")]
pub(crate) struct Cli {
    #[arg(long)]
    pub(crate) port: Option<u16>,
    #[arg(long)]
    pub(crate) max_connections: Option<usize>,
}
