use clap::{Parser, Subcommand};

/// A Merkle tree implementation for proving file integrity.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path where client files are located
    #[arg(short, long, default_value_t = String::from("client_files"))]
    pub files_path: String,

    /// Path where client computed merkle root is stored on disk
    #[arg(short, long, default_value_t = String::from("merkle.bin"))]
    pub merkle_path: String,

    /// Server IP address
    #[arg(short, long, default_value_t = String::from("http://127.0.0.1:3000"))]
    pub server_address: String,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Uploads all files to the server
    Upload {},

    /// Request a file by name
    Request { file_name: String },
}

impl Args {
    pub fn parse_arguments() -> Self {
        Args::parse()
    }
}
