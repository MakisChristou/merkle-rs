use axum::{
    extract::{Json, Path},
    handler::{get, post},
    Router,
};
use base64;
use hyper::Server;
use merkle_tree::ProofListItem;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{
    collections::BTreeMap,
    fs::{create_dir_all, File},
    io::Write,
};
use tower::ServiceBuilder;

mod args;
mod merkle_tree;
mod utils;

use clap::Parser;

use crate::{args::Args, merkle_tree::MerkleTree};

#[derive(Serialize, Deserialize)]
struct FileResponse {
    filename: String,
    content: Vec<u8>,
    merkle_proof: Vec<ProofListItem>,
}

#[derive(Deserialize)]
struct UploadPayload {
    filename: String,
    content: String,
}

pub struct MerkleServer {
    files_path: String,
    files: BTreeMap<String, Vec<u8>>,
    merkle_tree: MerkleTree,
}

impl MerkleServer {
    fn new(files_path: String, files: BTreeMap<String, Vec<u8>>) -> Self {
        let merkle_tree = MerkleTree::new(&files);
        MerkleServer {
            files_path,
            files,
            merkle_tree,
        }
    }
}

async fn upload(Json(body): Json<UploadPayload>) -> &'static str {
    // Create the directory if it doesn't exist
    let path = std::path::Path::new("server_files");
    if !path.exists() {
        if let Err(e) = create_dir_all(&path) {
            eprintln!("Failed to create directory: {:?}", e);
            return "Internal server error";
        }
    }

    // Decode the base64 content
    let content_bytes = match base64::decode(&body.content) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Failed to decode base64 content: {:?}", e);
            return "Invalid file content";
        }
    };

    // Save the file
    let file_path = path.join(&body.filename);
    if let Err(e) = File::create(&file_path).and_then(|mut file| file.write_all(&content_bytes)) {
        eprintln!("Failed to save file: {:?}", e);
        return "Internal server error";
    }

    "File uploaded successfully!"
}

// Endpoint to retrieve a file by name.
async fn get_file(Path(filename): Path<String>) -> String {
    // Placeholder: In a real scenario, you'd fetch the file and its Merkle proof from storage.
    format!(
        "File: {}\nContent: {}\nMerkle Proof: {}",
        filename, "Sample file content.", "Sample merkle proof."
    )
}

#[tokio::main]
async fn main() {
    println!("Hello from server");

    let app = Router::new()
        .route("/upload", post(upload))
        .route("/file/:filename", get(get_file));

    let addr = "127.0.0.1:3000".parse().unwrap();

    let server = Server::bind(&addr).serve(app.into_make_service());

    println!("Server running on http://{}", addr);

    server.await.expect("Server error");
}
