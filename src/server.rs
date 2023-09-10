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

async fn request_file(
    directory: String,
    Path(filename): Path<String>,
) -> Result<Json<FileResponse>, StatusCode> {
    let file_path = format!("{}/{}", directory, filename);

    let content = match fs::read(file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read file {}/{}: {:?}", directory, filename, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let files = utils::parse_files(&directory);
    let merkle_tree = MerkleTree::new(&files);

    match merkle_tree.generate_merkle_proof(&filename, &files) {
        Some(proof_list) => Ok(Json(FileResponse::new(filename, content, proof_list))),
        None => {
            eprintln!(
                "Failed to generate merkle proof for {}/{}",
                directory, filename
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
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
            post({
                let directory = directory.clone();
                move |body: Json<UploadRequest>| upload(directory.clone(), body)
            }),
        )
        .route(
            "/file/:filename",
            get({
                let directory = directory.clone();
                move |filename: Path<String>| request_file(directory.clone(), filename)
            }),
        );

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], args.port));

    println!("Welcome to merkle-rs server 🔑🦀!");
    println!("Listening on {}", addr);

    hyper::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Server failed to start");
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::{Body, Request};
    use tempfile::tempdir;
    use tokio_test::block_on;

    impl UploadRequest {
        pub fn from_req(req: Request<Body>) -> Result<Self, StatusCode> {
            let bytes = block_on(hyper::body::to_bytes(req.into_body())).unwrap();
            let body_str = String::from_utf8(bytes.to_vec()).unwrap();
            serde_json::from_str(&body_str).map_err(|_| StatusCode::BAD_REQUEST)
        }
    }

    fn mock_upload_request(content: &str, filename: &str) -> Request<Body> {
        let body = UploadRequest {
            content: content.to_string(),
            filename: filename.to_string(),
        };
        let body = serde_json::to_string(&body).unwrap();
        Request::builder()
            .method("POST")
            .uri("/upload")
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap()
    }

    fn upload_two_files(directory: String) {
        let req = mock_upload_request("SGVsbG8gV29ybGQ=", "hello1.txt");
        let _ = block_on(upload(
            directory.clone(),
            Json(UploadRequest::from_req(req).unwrap()),
        ));

        let req = mock_upload_request("SGVsbG8gV29ybGQ=", "hello2.txt");
        let _: Result<Json<UploadResponse>, StatusCode> = block_on(upload(
            directory,
            Json(UploadRequest::from_req(req).unwrap()),
        ));
    }

    #[test]
    fn test_upload() {
        let dir = tempdir().unwrap();
        let directory = dir.path().to_str().unwrap().to_string();
        let req = mock_upload_request("SGVsbG8gV29ybGQ=", "hello.txt");
        let resp = block_on(upload(
            directory.clone(),
            Json(UploadRequest::from_req(req).unwrap()),
        ));

        assert!(resp.is_ok());
        assert_eq!(
            resp.unwrap().0.message,
            "File uploaded succesfully".to_string()
        );
    }

    #[test]
    fn test_request_file() {
        let dir = tempdir().unwrap();
        let directory = dir.path().to_str().unwrap().to_string();
        upload_two_files(directory.clone());

        let filename = "hello1.txt".to_string();
        let resp = block_on(request_file(directory, Path(filename)));

        assert!(resp.is_ok());
        let file_response = resp.unwrap().0;
        assert_eq!(file_response.filename, "hello1.txt".to_string());
        assert_eq!(
            file_response.content,
            vec![72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100]
        ); // "Hello World" in bytes
    }
}
