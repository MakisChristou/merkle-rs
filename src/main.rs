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

    // Get the hash of a specific file (e.g., "file1") to use as the target
    let target_hash = Sha256::digest(&files["file8.txt"]).to_vec();

    println!("target_hash: {}", hex::encode(&target_hash));

    // Call the find_target_relative_to_node function
    let result = merkle_tree
        .find_target_relative_to_node(&merkle_tree.root.right.clone().unwrap(), &target_hash);

    // Print the result
    match result {
        Some(NodeOrder::Left) => println!("The target is in the left subtree of the root."),
        Some(NodeOrder::Right) => println!("The target is in the right subtree of the root."),
        None => println!("The target is not a descendant of the root."),
    }
}
