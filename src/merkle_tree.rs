use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct MerkleNode {
    pub hash: Vec<u8>,
    left: Option<Box<MerkleNode>>,
    right: Option<Box<MerkleNode>>,
}

#[derive(Clone, Debug)]
pub enum NodeOrder {
    Right,
    Left,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct ProofListItem {
    hash: Vec<u8>,
    order: NodeOrder,
}

impl MerkleNode {
    fn new(data: &Vec<u8>) -> Self {
        let hash = Sha256::digest(data).to_vec();
        MerkleNode {
            hash,
            left: None,
            right: None,
        }
    }

    pub fn combine(left: &MerkleNode, right: &MerkleNode) -> Self {
        let combined = [&left.hash[..], &right.hash[..]].concat();
        let hash = Sha256::digest(&combined).to_vec();
        MerkleNode {
            hash,
            left: Some(Box::new(left.clone())),
            right: Some(Box::new(right.clone())),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MerkleTree {
    pub root: MerkleNode,
}

use hex;
use std::fmt;

impl fmt::Display for MerkleNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Helper function to recursively print nodes with indentation
        fn print_node(node: &MerkleNode, f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
            // Convert the first few bytes of the hash to a hex string for brevity
            let short_hash = hex::encode(&node.hash[0..3]);

            // Recursively print left child first
            if let Some(left) = &node.left {
                print_node(left, f, depth + 1)?;
            }

            // Print the current node's hash with proper indentation
            writeln!(f, "{}{}", "        ".repeat(depth), short_hash)?;

            // Recursively print right child
            if let Some(right) = &node.right {
                print_node(right, f, depth + 1)?;
            }
            Ok(())
        }

        print_node(self, f, 0)
    }
}

impl fmt::Display for MerkleTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.root)
    }
}

impl MerkleTree {
    pub fn new(files: &BTreeMap<String, Vec<u8>>) -> Self {
        let mut nodes = Vec::new();
        for file in files {
            nodes.push(MerkleNode::new(file.1));
        }
        while nodes.len() > 1 {
            let mut next_level = Vec::new();
            while let Some(left) = nodes.pop() {
                if let Some(right) = nodes.pop() {
                    next_level.push(MerkleNode::combine(&left, &right));
                } else {
                    next_level.push(left);
                }
            }
            nodes = next_level;
        }
        MerkleTree {
            root: nodes.pop().unwrap(),
        }
    }

    pub fn get_root_hash(&self) -> Vec<u8> {
        self.root.hash.clone()
    }

    pub fn verify_merkle_proof(
        &self,
        proof_list: Vec<ProofListItem>,
        markle_root: &Vec<u8>,
    ) -> bool {
        todo!()
    }

    pub fn generate_merkle_proof(
        &self,
        file_name: &str,
        files: &BTreeMap<String, Vec<u8>>,
    ) -> Option<Vec<ProofListItem>> {
        todo!()
    }
}
