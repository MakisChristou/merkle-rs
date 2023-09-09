use std::collections::BTreeMap;

mod args;
mod merkle_tree;
mod utils;

use clap::Parser;

use crate::{merkle_tree::MerkleTree, args::Args};

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

    let args = Args::parse();

    let path = &args.path;
    let files = utils::parse_files(path);

    let merkle_tree = MerkleTree::new(&files);

    println!("\n{}", merkle_tree);

    match merkle_tree.generate_merkle_proof("backup.db", &files) {
        Some(proof_list) => {
            println!(
                "proof is : {}",
                utils::verify_merkle_proof(proof_list, merkle_tree.get_root_hash())
            );
        }
        None => {
            println!("Could not generate proof")
        }
    }

}
