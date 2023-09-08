mod client;
mod merkle_tree;
mod server;
mod utils;

use merkle_tree::MerkleTree;

fn main() {
    let files = (1..=8)
        .map(|i| {
            let file_name = format!("file{}.txt", i);
            let file_content = format!("File {} contents", i).into_bytes();
            (file_name, file_content)
        })
        .collect();

    let merkle_tree = MerkleTree::new(&files);

    println!("\n{}", merkle_tree);

    match merkle_tree.generate_merkle_proof("file1.txt", &files) {
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
