use axum::{
    extract::{Json, Path},
    handler::{get, post},
    Router,
};
use base64::{self, engine::general_purpose, Engine};
use clap::Parser;
use hyper::StatusCode;
use std::{
    fs::{self, create_dir_all, File},
    io::Write,
};

mod common;
mod merkle_tree;
mod server_args;
mod utils;

use crate::merkle_tree::MerkleTree;
use crate::{
    common::{FileResponse, UploadRequest, UploadResponse},
    server_args::Args,
};

async fn upload(
    directory: String,
    Json(body): Json<UploadRequest>,
) -> Result<Json<UploadResponse>, StatusCode> {
    // Create the directory if it doesn't exist
    let path = std::path::Path::new(&directory);
    if !path.exists() {
        if let Err(e) = create_dir_all(path) {
            eprintln!("Failed to create directory: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // Decode the base64 content
    let content_bytes = match general_purpose::STANDARD.decode(&body.content) {
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
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let directory = args.path;

    let app = Router::new()
        .route(
            "/upload",
            post(move |body: Json<UploadRequest>| upload(directory.clone(), body)),
        )
        .route("/file/:filename", get(request_file));

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], args.port));

    println!("Welcome to merkle-rs server 🔑🦀!");
    println!("Listening on {}", addr);

    hyper::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Server failed to start");
}
