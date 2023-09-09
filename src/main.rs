mod client;
mod merkle_tree;
mod server;
mod utils;
mod args;

use clap::Parser;
use merkle_tree::MerkleTree;
use crate::args::Args;

fn main() {

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
