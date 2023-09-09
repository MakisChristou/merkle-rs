use clap::Parser;

use crate::{args::Args, merkle_tree::MerkleTree};

use base64;
use reqwest;
use std::fs;

#[derive(serde::Serialize)]
struct UploadPayload {
    filename: String,
    content: String,
}

mod args;
mod merkle_tree;
mod utils;

pub struct Client {
    root_hash: Vec<u8>,
}

impl Client {
    pub fn new(root_hash: Vec<u8>) -> Self {
        Client { root_hash }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let base_url = "http://127.0.0.1:3000/upload";
    
    let entries = fs::read_dir("client_files")?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let content = fs::read(&path)?;
            let base64_content = base64::encode(&content);

            let payload = UploadPayload {
                filename: path.file_name().unwrap().to_string_lossy().into_owned(),
                content: base64_content,
            };

            let response = client.post(base_url)
                .json(&payload)
                .send()
                .await?;

            if response.status().is_success() {
                println!("Successfully uploaded: {:?}", path);
            } else {
                eprintln!("Failed to upload: {:?}. Status: {}", path, response.status());
            }
        }
    }

    Ok(())
}
