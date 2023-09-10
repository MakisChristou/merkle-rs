use crate::{
    client_args::{Args, Commands},
    merkle_tree::MerkleTree,
};
use base64::{self, engine::general_purpose, Engine};
use hyper::StatusCode;

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
    client_files: Option<String>,
    merkle_root_path: String,
}

static NO_DIR_MSG: &str = "Client has no set directory path";

impl MerkleClient {
    pub fn new(
        server_url: &str,
        reqwest_client: reqwest::Client,
        client_files: Option<String>,
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
                let response_body = response.text().await?;
                let file_response: FileResponse = serde_json::from_str(&response_body)?;
                Ok(file_response)
            }
            _ => Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Failed to retrieve file from server",
            ))),
        }
    }

    pub async fn upload_all_files_to_server(&self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.client_files {
            Some(client_files) => {
                let base_url = format!("{}/upload", self.server_url);

                let entries: Vec<_> = fs::read_dir(client_files)?.collect();
                if entries.is_empty() {
                    eprintln!("The directory is empty!");
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "The directory is empty",
                    )));
                } else if entries.len() < 2 {
                    eprintln!("Not enough files to upload, must be > 2");
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Not enough files to upload, must be > 2",
                    )));
                }

                let entries = fs::read_dir(client_files)?;

                for entry in entries {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file() {
                        MerkleClient::upload_file(&self.reqwest_client, &path, &base_url).await?;
                    }
                }

                Ok(())
            }
            None => {
                eprintln!("{}", NO_DIR_MSG);
                Err(Box::new(io::Error::new(io::ErrorKind::Other, NO_DIR_MSG)))
            }
        }
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
        match &self.client_files {
            Some(client_files) => {
                let entries = fs::read_dir(client_files)?;

                for entry in entries {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file() {
                        fs::remove_file(path)?;
                    }
                }

                Ok(())
            }
            None => {
                eprintln!("{}", NO_DIR_MSG);
                Err(io::Error::new(ErrorKind::Other, NO_DIR_MSG))
            }
        }
    }

    pub fn compute_merkle_root_from_files(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.client_files {
            Some(client_files) => {
                let files = utils::parse_files(client_files);

                if files.len() < 2 {
                    eprintln!("Not enough files to upload, must be > 2");
                    Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Not enough files to upload, must be > 2",
                    )))
                } else {
                    let merkle_tree = MerkleTree::new(&files);
                    self.merkle_root = Some(merkle_tree.root.hash);
                    Ok(())
                }
            }
            None => {
                eprintln!("{}", NO_DIR_MSG);
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    NO_DIR_MSG,
                )))
            }
        }
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
    let args = Args::parse_arguments();

    match &args.command {
        None => {
            println!("Welcome to merkle-rs client 🔑🦀!");
            println!("Please give a valid command");
            println!("Run with --help to get the list of available commands");
        }
        Some(Commands::Upload {}) => {
            let mut merkle_client = MerkleClient::new(
                &args.server_address,
                reqwest::Client::new(),
                Some(args.files_path),
                args.merkle_path,
            );

            match merkle_client.upload_all_files_to_server().await {
                Ok(_) => {
                    println!("Client files sucesfully uploaded!");
                }
                Err(e) => {
                    panic!("Could not upload files {}", e);
                }
            }

            if let Err(e) = merkle_client.compute_merkle_root_from_files() {
                panic!("Failed to compute merkle root from files {}", e);
            }

            if let Err(e) = merkle_client.write_merkle_root_to_disk() {
                panic!("Failed to write merkle root to disk {}", e);
            }

            if let Err(e) = merkle_client.delete_local_client_files() {
                panic!("Failed to delete client files {}", e);
            }
        }

        Some(Commands::Request { file_name }) => {
            let merkle_client = MerkleClient::new(
                &args.server_address,
                reqwest::Client::new(),
                None,
                args.merkle_path,
            );

            match merkle_client.request_file(file_name).await {
                Ok(server_response) => match merkle_client.read_merkle_root_from_disk() {
                    Ok(client_merkle_root) => {
                        if utils::verify_merkle_proof(
                            server_response.merkle_proof,
                            client_merkle_root,
                            server_response.content,
                        ) {
                            println!("Server proof is valid!");
                        } else {
                            eprintln!("Server proof is invalid!");
                        }
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                },
                Err(e) => {
                    eprint!("{}", e);
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialization() {
        let client = MerkleClient::new(
            "http://localhost:8000",
            reqwest::Client::new(),
            Some("path/to/files".to_owned()),
            "path/to/merkle_root".to_owned(),
        );
        assert_eq!(client.server_url, "http://localhost:8000");
        assert_eq!(client.client_files, Some("path/to/files".to_owned()));
        assert_eq!(client.merkle_root_path, "path/to/merkle_root".to_owned());
    }

    #[test]
    fn test_read_write_merkle_root() {
        let temp_dir = tempfile::tempdir().unwrap();
        let merkle_path = temp_dir.path().join("merkle_root");
        let mut client = MerkleClient::new(
            "http://localhost:8000",
            reqwest::Client::new(),
            Some("path/to/files".to_owned()),
            merkle_path.to_str().unwrap().to_string(),
        );

        client.merkle_root = Some(vec![1, 2, 3, 4]);
        let write_result = client.write_merkle_root_to_disk();
        assert!(write_result.is_ok());

        let read_result = client.read_merkle_root_from_disk().unwrap();
        assert_eq!(read_result, vec![1, 2, 3, 4]);
    }
}
