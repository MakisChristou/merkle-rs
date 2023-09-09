use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;

use crate::merkle_tree::{NodeOrder, ProofListItem};

pub fn parse_files(path: &str) -> BTreeMap<String, Vec<u8>> {
    let mut files_map = BTreeMap::new();
    let entries = fs::read_dir(path).expect("Failed to read directory");

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            // Extract the file name.
            if let Some(file_name) = path.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    // Read the file contents.
                    let mut file = fs::File::open(&path).expect("Failed to open file");
                    let mut content = Vec::new();
                    file.read_to_end(&mut content)
                        .expect("Failed to read file contents");

                    // Insert into the BTreeMap.
                    files_map.insert(file_name_str.to_string(), content);
                }
            }
        }
    }

    files_map
}

pub fn closest_bigger_power_of_two(n: u32) -> u32 {
    let log_value = (n as f64).log2();
    let ceil_value = log_value.ceil() as u32;
    2u32.pow(ceil_value)
}

fn contains_hash(proof_list: &Vec<ProofListItem>, target_hash: &Vec<u8>) -> bool {
    for item in proof_list {
        if &item.hash == target_hash {
            return true;
        }
    }
    false
}

pub fn verify_merkle_proof(
    mut proof_list: Vec<ProofListItem>,
    markle_root: Vec<u8>,
    file_contents: Vec<u8>,
) -> bool {
    if proof_list.len() < 2 {
        return false;
    }

    let hashed_file_contents = Sha256::digest(&file_contents).to_vec();

    if !contains_hash(&proof_list, &hashed_file_contents) {
        return false;
    }

    while proof_list.len() != 1 {
        let h1 = proof_list.pop().unwrap();
        let h2 = proof_list.pop().unwrap();

        match h2.order {
            Some(NodeOrder::Left) => {
                let combined = [&h2.hash[..], &h1.hash[..]].concat();
                let hash = Sha256::digest(&combined).to_vec();
                proof_list.push(ProofListItem::new(hash, None));
            }
            Some(NodeOrder::Right) => {
                let combined = [&h1.hash[..], &h2.hash[..]].concat();
                let hash = Sha256::digest(&combined).to_vec();
                proof_list.push(ProofListItem::new(hash, None));
            }
            None => {
                panic!("Should not be None");
            }
        }
    }

    markle_root == proof_list.pop().unwrap().hash
}
