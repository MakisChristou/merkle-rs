use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, rc::Rc};

#[derive(Clone, Debug)]
pub struct MerkleNode {
    pub hash: Vec<u8>,
    pub left: Option<Rc<MerkleNode>>,
    pub right: Option<Rc<MerkleNode>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NodeOrder {
    Right,
    Left,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ProofListItem {
    pub hash: Vec<u8>,
    pub order: Option<NodeOrder>,
}

impl ProofListItem {
    pub fn new(hash: Vec<u8>, order: Option<NodeOrder>) -> Self {
        ProofListItem { hash, order }
    }
}

impl fmt::Debug for ProofListItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let hash_str = hex::encode(&self.hash[0..3]);
        match &self.order {
            Some(NodeOrder::Right) => write!(f, "h: {} Right", hash_str),
            Some(NodeOrder::Left) => write!(f, "h: {} Left", hash_str),
            None => write!(f, "h: {}", hash_str),
        }
    }
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
        let hash = Sha256::digest(combined).to_vec();
        MerkleNode {
            hash,
            left: Some(Rc::new(left.clone())),
            right: Some(Rc::new(right.clone())),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MerkleTree {
    pub root: MerkleNode,
}

use std::fmt;

use crate::utils;

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

        // Balance tree
        if !nodes.len().is_power_of_two() {
            let last_item = nodes.last().unwrap().clone();

            let times = utils::closest_bigger_power_of_two(nodes.len() as u32) - nodes.len() as u32;

            for _ in 0..times {
                nodes.push(last_item.clone());
            }
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

    pub fn generate_merkle_proof(
        &self,
        file_name: &str,
        files: &BTreeMap<String, Vec<u8>>,
    ) -> Option<Vec<ProofListItem>> {
        let mut current_node = Some(Rc::new(self.root.clone()));

        let file_contents = files.get(file_name)?;
        let target_hash: Vec<u8> = Sha256::digest(file_contents).to_vec();
        let mut proof_list: Vec<ProofListItem> = Vec::new();

        while let Some(ref node) = current_node {
            match self.find_target_relative_to_node(node.as_ref(), &target_hash) {
                Some(NodeOrder::Right) => {
                    if let Some(left) = &node.left {
                        if let Some(right) = &node.right {
                            // If we are at the end add both leaves (one is the target)
                            if left.hash == target_hash || right.hash == target_hash {
                                proof_list.push(ProofListItem::new(
                                    right.hash.clone(),
                                    Some(NodeOrder::Right),
                                ));
                            }

                            proof_list
                                .push(ProofListItem::new(left.hash.clone(), Some(NodeOrder::Left)));
                            current_node = Some(Rc::clone(right));
                        }
                    }
                }
                Some(NodeOrder::Left) => {
                    if let Some(right) = &node.right {
                        if let Some(left) = &node.left {
                            // If we are at the end add both leaves (one is the target)
                            if left.hash == target_hash || right.hash == target_hash {
                                proof_list.push(ProofListItem::new(
                                    left.hash.clone(),
                                    Some(NodeOrder::Left),
                                ));
                            }

                            proof_list.push(ProofListItem::new(
                                right.hash.clone(),
                                Some(NodeOrder::Right),
                            ));
                            current_node = Some(Rc::clone(left));
                        }
                    }
                }
                None => break,
            }
        }

        if proof_list.is_empty() {
            return None;
        }

        Some(proof_list)
    }

    pub fn find_target_relative_to_node(
        &self,
        node: &MerkleNode,
        target_hash: &Vec<u8>,
    ) -> Option<NodeOrder> {
        if MerkleTree::is_node_in_subtree(&node.left, target_hash) {
            Some(NodeOrder::Left)
        } else if MerkleTree::is_node_in_subtree(&node.right, target_hash) {
            Some(NodeOrder::Right)
        } else {
            None
        }
    }

    fn is_node_in_subtree(node: &Option<Rc<MerkleNode>>, target_hash: &Vec<u8>) -> bool {
        match node {
            Some(n) => {
                if &n.hash == target_hash {
                    return true;
                }
                MerkleTree::is_node_in_subtree(&n.left, target_hash)
                    || MerkleTree::is_node_in_subtree(&n.right, target_hash)
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        merkle_tree::{MerkleTree, NodeOrder},
        utils,
    };
    use sha2::{Digest, Sha256};
    use std::collections::BTreeMap;

    fn setup_test() -> (MerkleTree, BTreeMap<String, Vec<u8>>) {
        let files = (1..=8)
            .map(|i| {
                let file_name = format!("file{}.txt", i);
                let file_content = format!("File {} contents", i).into_bytes();
                (file_name, file_content)
            })
            .collect();

        let merkle_tree = MerkleTree::new(&files);

        (merkle_tree, files)
    }

    #[test]
    fn should_find_left_target_relative_to_node() {
        let (merkle_tree, files) = setup_test();

        let target_hash = Sha256::digest(&files["file8.txt"]).to_vec();

        let result =
            merkle_tree.find_target_relative_to_node(&merkle_tree.root.clone(), &target_hash);

        assert_eq!(result, Some(NodeOrder::Left));
    }

    #[test]
    fn should_find_right_target_relative_to_node() {
        let (merkle_tree, files) = setup_test();

        let target_hash = Sha256::digest(&files["file3.txt"]).to_vec();

        let result =
            merkle_tree.find_target_relative_to_node(&merkle_tree.root.clone(), &target_hash);

        assert_eq!(result, Some(NodeOrder::Right));
    }

    #[test]
    fn should_not_find_target_if_not_exist() {
        let (merkle_tree, files) = setup_test();

        let target_hash = Sha256::digest(&files["file8.txt"]).to_vec();

        let result = merkle_tree
            .find_target_relative_to_node(&merkle_tree.root.clone().right.unwrap(), &target_hash);

        assert_eq!(result, None);
    }

    #[test]
    fn should_accept_correct_proof() {
        let (merkle_tree, files) = setup_test();

        match merkle_tree.generate_merkle_proof("file1.txt", &files) {
            Some(proof_list) => {
                assert!(utils::verify_merkle_proof(
                    proof_list,
                    merkle_tree.get_root_hash()
                ));
            }
            None => {
                assert!(false);
            }
        }
    }

    #[test]
    fn should_reject_invalid_proof() {
        let (merkle_tree, files) = setup_test();

        let modified_files: BTreeMap<String, Vec<u8>> = vec![
            (
                "file1.txt".to_string(),
                b"Modified File 1 contents".to_vec(),
            ),
            (
                "file2.txt".to_string(),
                b"Modified File 2 contents".to_vec(),
            ),
            (
                "file3.txt".to_string(),
                b"Modified File 3 contents".to_vec(),
            ),
            (
                "file4.txt".to_string(),
                b"Modified File 4 contents".to_vec(),
            ),
            (
                "file5.txt".to_string(),
                b"Modified File 5 contents".to_vec(),
            ),
            (
                "file6.txt".to_string(),
                b"Modified File 6 contents".to_vec(),
            ),
            (
                "file7.txt".to_string(),
                b"Modified File 7 contents".to_vec(),
            ),
            (
                "file8.txt".to_string(),
                b"Modified File 8 contents".to_vec(),
            ),
        ]
        .into_iter()
        .collect();

        let modified_merkle_tree = MerkleTree::new(&modified_files);

        match modified_merkle_tree.generate_merkle_proof("file1.txt", &modified_files) {
            Some(proof_list) => {
                assert!(!utils::verify_merkle_proof(
                    proof_list,
                    merkle_tree.get_root_hash()
                ));
            }
            None => {
                assert!(false);
            }
        }
    }
}
