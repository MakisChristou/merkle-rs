use merkle_tree::ProofListItem;
use serde::{Deserialize, Serialize};

use crate::merkle_tree;

#[derive(Serialize, Deserialize)]
pub struct UploadRequest {
    pub filename: String,
    pub content: String,
}

#[derive(Deserialize, Serialize)]
pub struct UploadResponse {
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct FileResponse {
    pub filename: String,
    pub content: Vec<u8>,
    pub merkle_proof: Vec<ProofListItem>,
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
