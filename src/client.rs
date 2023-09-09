use crate::{client_args::Args, merkle_tree::MerkleTree};
use base64;
use clap::Parser;
use reqwest;
use std::{
    fs,
    io::{self, ErrorKind},
};

#[derive(serde::Serialize)]
struct UploadPayload {
    filename: String,
    content: String,
}

mod client_args;
mod merkle_tree;
mod utils;

pub struct MerkleClient {
    root_hash: Option<Vec<u8>>,
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
            root_hash: None,
            server_url: server_url.to_owned(),
            reqwest_client,
            client_files,
            merkle_root_path,
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
        match self.root_hash.clone() {
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
        self.root_hash = Some(merkle_tree.root.hash)
    }

    async fn upload_file(
        client: &reqwest::Client,
        path: &std::path::Path,
        base_url: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read(&path)?;
        let base64_content = base64::encode(&content);

        let payload = UploadPayload {
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
    merkle_client.write_merkle_root_to_disk();
    merkle_client.delete_local_client_files();

    Ok(())
}
