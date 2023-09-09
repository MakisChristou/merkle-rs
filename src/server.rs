use axum::{
    extract::{Json, Path},
    handler::{get, post},
    Router,
};
use base64;
use hyper::StatusCode;
use merkle_tree::ProofListItem;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, create_dir_all, File},
    io::Write,
};

mod args;
mod merkle_tree;
mod utils;

use crate::merkle_tree::MerkleTree;

#[derive(Serialize, Deserialize)]
struct FileResponse {
    filename: String,
    content: Vec<u8>,
    merkle_proof: Vec<ProofListItem>,
}

impl FileResponse {
    pub fn new(filename: String, content: Vec<u8>, merkle_proof: Vec<ProofListItem>) -> Self {
        FileResponse {
            filename,
            content,
            merkle_proof,
        }
    }
}

#[derive(Deserialize, Serialize)]
struct UploadPayload {
    filename: String,
    content: String,
}

#[derive(Deserialize, Serialize)]
struct UploadResponse {
    message: String,
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

async fn upload(Json(body): Json<UploadPayload>) -> Result<Json<UploadResponse>, StatusCode> {
    // Create the directory if it doesn't exist
    let path = std::path::Path::new("server_files");
    if !path.exists() {
        if let Err(e) = create_dir_all(&path) {
            eprintln!("Failed to create directory: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // Decode the base64 content
    let content_bytes = match base64::decode(&body.content) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Failed to decode base64 content: {:?}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // Save the file
    let file_path = path.join(&body.filename);
    if let Err(e) = File::create(&file_path).and_then(|mut file| file.write_all(&content_bytes)) {
        eprintln!("Failed to save file: {:?}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(UploadResponse {
        message: "File uploaded succesfully".to_owned(),
    }))
}

async fn get_file(Path(filename): Path<String>) -> Result<Json<FileResponse>, StatusCode> {
    let path = &"server_files";

    // Read the file's content
    let content = match fs::read(&path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read file {}: {:?}", filename, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let files = utils::parse_files(path);

    let merkle_tree = MerkleTree::new(&files);

    match merkle_tree.generate_merkle_proof("backup.db", &files) {
        Some(proof_list) => Ok(Json(FileResponse::new(filename, content, proof_list))),
        None => {
            panic!("Server could not generate proof")
        }
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/upload", post(upload))
        .route("/file/:filename", get(get_file));

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on {}", addr);
    hyper::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Server failed to start");
}
