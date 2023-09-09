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
}
