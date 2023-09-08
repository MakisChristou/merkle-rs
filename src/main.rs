mod merkle_tree;
mod utils;

use std::collections::BTreeMap;

use merkle_tree::MerkleTree;
use sha2::{Digest, Sha256};

use crate::merkle_tree::NodeOrder;

struct Client {
    root_hash: Vec<u8>,
}

struct Server {
    files: BTreeMap<String, Vec<u8>>,
    merkle_tree: MerkleTree,
}

impl Client {
    fn new(root_hash: Vec<u8>) -> Self {
        Client { root_hash }
    }
}

impl Server {
    fn new(files: BTreeMap<String, Vec<u8>>) -> Self {
        let merkle_tree = MerkleTree::new(&files);
        Server { files, merkle_tree }
    }
}

fn main() {
    let files = vec![
        ("file1.txt".to_string(), b"File 1 contents".to_vec()),
        ("file2.txt".to_string(), b"File 2 contents".to_vec()),
        ("file3.txt".to_string(), b"File 3 contents".to_vec()),
        ("file4.txt".to_string(), b"File 4 contents".to_vec()),
        ("file5.txt".to_string(), b"File 5 contents".to_vec()),
        ("file6.txt".to_string(), b"File 6 contents".to_vec()),
        ("file7.txt".to_string(), b"File 7 contents".to_vec()),
        ("file8.txt".to_string(), b"File 8 contents".to_vec()),
    ]
    .into_iter()
    .collect();

    let merkle_tree = MerkleTree::new(&files);

    println!("{}", merkle_tree);

    let proof_list = merkle_tree
        .generate_merkle_proof("file1.txt", &files)
        .unwrap();

    println!("proof_list: {:?}", proof_list);

    println!(
        "Is proof valid: {}",
        merkle_tree.verify_merkle_proof(proof_list, merkle_tree.get_root_hash())
    );
}
