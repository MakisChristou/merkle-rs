use axum::{
    extract::{Json, Path},
    handler::{get, post},
    Router,
};
use base64;
use hyper::StatusCode;
use std::{
    collections::BTreeMap,
    fs::{self, create_dir_all, File},
    io::Write,
};

mod common;
mod merkle_tree;
mod server_args;
mod utils;

use crate::common::{FileResponse, UploadRequest, UploadResponse};
use crate::merkle_tree::MerkleTree;

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

async fn upload(Json(body): Json<UploadRequest>) -> Result<Json<UploadResponse>, StatusCode> {
    // Create the directory if it doesn't exist
    let path = std::path::Path::new("server_files");
    if !path.exists() {
        if let Err(e) = create_dir_all(path) {
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
    if let Err(e) = File::create(file_path).and_then(|mut file| file.write_all(&content_bytes)) {
        eprintln!("Failed to save file: {:?}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(UploadResponse {
        message: "File uploaded succesfully".to_owned(),
    }))
}

async fn request_file(Path(filename): Path<String>) -> Result<Json<FileResponse>, StatusCode> {
    let file_path = format!("server_files/{}", filename);

    let path = "server_files";

    let content = match fs::read(file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read file {}: {:?}", filename, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let files = utils::parse_files(path);
    let merkle_tree = MerkleTree::new(&files);

    match merkle_tree.generate_merkle_proof(&filename, &files) {
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
        .route("/file/:filename", get(request_file));

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on {}", addr);
    hyper::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Server failed to start");
}
