use std::collections::BTreeMap;

use crate::merkle_tree::MerkleTree;

pub struct Server {
    files: BTreeMap<String, Vec<u8>>,
    merkle_tree: MerkleTree,
}

impl Server {
    fn new(files: BTreeMap<String, Vec<u8>>) -> Self {
        let merkle_tree = MerkleTree::new(&files);
        Server { files, merkle_tree }
    }
}
