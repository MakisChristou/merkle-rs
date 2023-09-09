use std::collections::BTreeMap;

mod args;
mod merkle_tree;
mod utils;

use crate::merkle_tree::MerkleTree;

pub struct Server {
    files_path: String,
    files: BTreeMap<String, Vec<u8>>,
    merkle_tree: MerkleTree,
}

impl Server {
    fn new(files_path: String, files: BTreeMap<String, Vec<u8>>) -> Self {
        let merkle_tree = MerkleTree::new(&files);
        Server {
            files_path,
            files,
            merkle_tree,
        }
    }
}

fn main() {
    println!("Hello from server!");
}
