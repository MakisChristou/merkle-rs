use clap::Parser;

/// A Merkle tree implementation for proving file integrity.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path where client files are located
    #[arg(short, long, default_value_t = String::from("target_files"))]
    pub path: String,
}
