use clap::Parser;

/// A Merkle Tree implementation for proving file integrity.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path where client files are located
    #[arg(long, default_value_t = String::from("server_files"))]
    pub path: String,

    /// Port to listen to
    #[arg(long, default_value_t = 3000)]
    pub port: u16,
}
