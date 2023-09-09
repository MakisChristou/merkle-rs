use crate::{client_args::Args, merkle_tree::MerkleTree};
use base64::{self, engine::general_purpose, Engine};
use clap::Parser;
use hyper::StatusCode;

use reqwest;
use std::{
    fs,
    io::{self, ErrorKind},
};

mod client_args;
mod common;
mod merkle_tree;
mod utils;

use common::{FileResponse, UploadRequest};

pub struct MerkleClient {
    pub merkle_root: Option<Vec<u8>>,
    server_url: String,
    reqwest_client: reqwest::Client,
    client_files: String,
    merkle_root_path: String,
}

impl MerkleClient {
    pub fn new(
        server_url: &str,
        reqwest_client: reqwest::Client,
        client_files: String,
        merkle_root_path: String,
    ) -> Self {
        MerkleClient {
            merkle_root: None,
            server_url: server_url.to_owned(),
            reqwest_client,
            client_files,
            merkle_root_path,
        }
    }
    pub async fn request_file(
        &self,
        filename: &str,
    ) -> Result<FileResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/file/{}", &self.server_url, filename);

        let response = self.reqwest_client.get(&url).send().await?;

        match response.status() {
            StatusCode::OK => {
                let file_response: FileResponse = response.json().await?;
                Ok(file_response)
            }
            _ => Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Failed to retrieve file from server",
            ))),
        }
    }

    pub async fn upload_all_files_to_server(&self) -> Result<(), Box<dyn std::error::Error>> {
        let base_url = format!("{}/upload", self.server_url);
        let entries = fs::read_dir(self.client_files.clone())?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                MerkleClient::upload_file(&self.reqwest_client, &path, &base_url).await?;
            }
        }

        Ok(())
    }

    pub fn read_merkle_root_from_disk(&self) -> io::Result<Vec<u8>> {
        fs::read(self.merkle_root_path.clone())
    }

    pub fn write_merkle_root_to_disk(&self) -> io::Result<()> {
        match self.merkle_root.clone() {
            Some(merkle_root) => fs::write(self.merkle_root_path.clone(), merkle_root),
            None => {
                eprintln!("Client has no merkle root to store");
                Err(io::Error::new(
                    ErrorKind::Other,
                    "Client has no merkle root to store",
                ))
            }
        }
    }

    pub fn delete_local_client_files(&self) -> io::Result<()> {
        let entries = fs::read_dir(&self.client_files)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                fs::remove_file(path)?;
            }
        }

        Ok(())
    }

    pub fn compute_merkle_root_from_files(&mut self) {
        let files = utils::parse_files(&self.client_files);
        let merkle_tree = MerkleTree::new(&files);
        self.merkle_root = Some(merkle_tree.root.hash)
    }

    async fn upload_file(
        client: &reqwest::Client,
        path: &std::path::Path,
        base_url: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read(path)?;
        let base64_content = general_purpose::STANDARD.encode(&content);

        let payload = UploadRequest {
            filename: path.file_name().unwrap().to_string_lossy().into_owned(),
            content: base64_content,
        };

        let response = client.post(base_url).json(&payload).send().await?;

        if response.status().is_success() {
            println!("Successfully uploaded: {:?}", path);
        } else {
            eprintln!(
                "Failed to upload: {:?}. Status: {}",
                path,
                response.status()
            );
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut merkle_client = MerkleClient::new(
        "http://127.0.0.1:3000",
        reqwest::Client::new(),
        args.files_path,
        args.merkle_path,
    );

    match merkle_client.upload_all_files_to_server().await {
        Ok(_) => {
            println!("Client files sucesfully uploaded!");
        }
        Err(e) => {
            println!("Could not upload files {}", e);
        }
    }

    merkle_client.compute_merkle_root_from_files();
    if let Err(e) = merkle_client.write_merkle_root_to_disk() {
        panic!("Failed to write merkle root to disk {}", e);
    }

    if let Err(e) = merkle_client.delete_local_client_files() {
        panic!("Failed to delete client files {}", e);
    }

    println!("Requesting file from server");

    let file_to_ask_for = "file1.txt";

    match merkle_client.request_file(file_to_ask_for).await {
        Ok(server_response) => {
            if utils::verify_merkle_proof(
                server_response.merkle_proof,
                merkle_client.merkle_root.unwrap(),
                server_response.content,
            ) {
                println!("Server proof is valid!");
            } else {
                println!("Server proof is invalid!");
            }
        }
        Err(e) => {
            eprint!("{}", e);
        }
    }

    Ok(())
}
