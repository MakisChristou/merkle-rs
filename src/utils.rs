use sha2::{Digest, Sha256};

use crate::merkle_tree::{NodeOrder, ProofListItem};

pub fn hash_file(path: &str) -> Vec<u8> {
    todo!()
}

pub fn sort_files(path: &str) -> Vec<(String, Vec<u8>)> {
    todo!()
}

pub fn closest_bigger_power_of_two(n: u32) -> u32 {
    let log_value = (n as f64).log2();
    let ceil_value = log_value.ceil() as u32;
    2u32.pow(ceil_value)
}

pub fn verify_merkle_proof(mut proof_list: Vec<ProofListItem>, markle_root: Vec<u8>) -> bool {
    if proof_list.len() < 2 {
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

    return markle_root == proof_list.pop().unwrap().hash;
}
