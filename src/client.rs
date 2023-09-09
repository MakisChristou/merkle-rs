use clap::Parser;

use crate::{args::Args, merkle_tree::MerkleTree};

mod args;
mod merkle_tree;
mod utils;

pub struct Client {
    root_hash: Vec<u8>,
}

impl Client {
    pub fn new(root_hash: Vec<u8>) -> Self {
        Client { root_hash }
    }
}

fn main() {
    println!("Hello from client!");
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
