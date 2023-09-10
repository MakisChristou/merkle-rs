# merkle-rs
A Merkle tree implementation in Rust 🌲🦀! 

- Supports merkle proof generation/verification for a set of files
- Contains 2 networked binaries (a client and a server)
- Server API supports uploading and requesting files alongside their Merkle Proofs
- Configurable ports, directories for storing files and proofs
- Works with any file type
- Proof generation < 2 seconds on M2 Macbook pro for 100k small files

# How it works

## Merkle Tree
A Merkle Tree is a data structure that is used to efficienlty summarize a set of data (usually transactions in a blockchain). It is a binary tree where the leaf nodes contain the hashes of the files/transactions that we want to "summarize" and the parent nodes are computed by grouping leaf nodes into two and concatenating their hashes, rehashing them to get a combined hash. That hash is the value of the parent node. We do this recursively until we reach the root node which is the "summary" of the set of data. This enables an efficient way to check if a piece of data that you have the contents of is in this larger set of data (i.e. block in a blockchain context). The bread and butter of Merkle Trees is their efficient Proof generation and verification algoerithms. We will talk about them in the next sections.

## Merkle Proof Generation Algorithm (server)
The Merkle Proof generation algorithm can be found in the `generate_merkle_proof` method of `MerkleTree`. The idea behind the algorithm is to start from the root node (`current_node` = `root_of_tree`) of the tree and traverse downards. While doing so we keep a `proof_list` which is a stack that contains the hashes of the required nodes for the proof, alongside their order in the tree (left or right). We set the `current_node`  to the left child if the `target_hash` is found under the left subtree and we push the opposite (i.e. right child) in the `proof_list` alongside its order which in this case is right. We do the exact opposite if the `target_hash` is found in the opposite subtree. We continue this operation until we reach the lead nodes of the tree. A simplified pseudocode of the algorithm can be found below:

```python
def generate_merkle_proof(target_hash, root_of_tree):
    proof_list = []
    current_node = root_of_tree

    while not current_node.is_leaf():
        if target_hash in current_node.left_subtree():
            proof_list.push((current_node.right_child.hash, "right"))
            current_node = current_node.left_child
        else:
            proof_list.push((current_node.left_child.hash, "left"))
            current_node = current_node.right_child

    return proof_list

```

The `proof_list` alongside the contents of the file are what the verification algorithm needs to check if the given file is in the Merkle Tree.

## Merkle Proof Verification Algorithm (client)
I have chosen to implement the verification algorithm as a helper method in the `utils` module since it should be independent of the actual tree. Spefically the implementation is in the `utils::verify_merkle_proof` function. The verification algorithm is relatively simpler. Firstly there are some quick ways to dismiss an invalid proof such as checking if the `proof_list` is empty or below a given size, and then checking if the hashed contents of the file are included in any of the nodes in the `proof_list`. If that is the case, all the verifier has to do is pop items from the `proof_list`, specificslly 2 at a time, concatenate their hashes in the correct order, which is included in each item in the proof list and generate a new node which is to be pushed back to the stack. It then repeats this process until the stack size is 1. If the result is equal to the merkle tree's root hash then the verification is sucessful. A simplified pseudocode of the algorithm can be found below:

```python
def verify_merkle_proof(proof_list, hashed_file_contents, merkle_root):
    if not proof_list or len(proof_list) < THRESHOLD:
        return False

    if hashed_file_contents not in [node.hash for node in proof_list]:
        return False

    while len(proof_list) > 1:
        item1 = proof_list.pop()
        item2 = proof_list.pop()

        if item1.order == "left":
            concatenated_hash = hash_function(item1.hash + item2.hash)
        else:
            concatenated_hash = hash_function(item2.hash + item1.hash)

        proof_list.push(concatenated_hash)

    if len(proof_list) == 1 and proof_list[0] == merkle_root:
        return True
    else:
        return False

```




# Run tests 

```bash
$ cargo t
```

# Command Line Arguments

## Server Arguments
The server has 2 main configuration options. The port which it listens to as well as the path on disk where the client uploaded files will be stored. The default options are port 3000 and the directory `./server_files`.

```bash
$ cargo r --bin server -- --help
A Merkle tree implementation for proving file integrity

Usage: server [OPTIONS]

Options:
      --path <PATH>  Path where client files are located [default: server_files]
      --port <PORT>  Port to listen to [default: 3000]
  -h, --help         Print help
  -V, --version      Print version
```

## Client Arguments
The client has multiple configuration options. For instance we can choose the path of the directory where the files will be uploaded from and deleted, the path of the merkle root and the server address and port.


```bash
$ client --help
A Merkle tree implementation for proving file integrity

Usage: client [OPTIONS] [COMMAND]

Commands:
  upload   Uploads all files to the server
  request  Request a file by name
  help     Print this message or the help of the given subcommand(s)

Options:
  -f, --files-path <FILES_PATH>
          Path where client files are located [default: client_files]
  -m, --merkle-path <MERKLE_PATH>
          Path where client computed merkle root is stored on disk [default: merkle.bin]
  -s, --server-address <SERVER_ADDRESS>
          Server IP address [default: http://127.0.0.1:3000]
  -h, --help
          Print help
  -V, --version
          Print version
```

# Run using Docker
Before running our first client command we need to create some files locally on the machine where the client is running. For simplicity we can create a new folder called `client_files` and add some simple text files in there for example's sake. I have added a helper script to do just that, which essentially creates a number of text files which is given as a cli argument under `client_files`.

```bash
./gen_files.sh 10000
```

Start the server container with on port 3000

```bash
docker-compose run server cargo r --release --bin server -- --port 3000
```

Once the server is built and up and running start the client with the upload command. This will upload the generated files of our helper script to the server, generate a merkle proof and store it under `merkle.bin` and then delete all of the client files.

```bash
docker-compose run client cargo r --release --bin client -- --server-address="http://server:3000" upload
```

Then we can request a specific file with its proof from the server


```bash
docker-compose run client cargo r --release --bin client -- --server-address="http://server:3000" request "file1.txt"
```

# Limitations/Shortcomings
- Files and their content are stored in RAM when constructing the Merkle Tree (impractical for larger files)
- Cannot incrementally upload files
- If a new set of files is sent Merkle Tree should be re-created from scratch
- No user authentication, anyone can request or upload a file from/to the server
- Files are sent in plain-text
- Client can only upload all of its files found under a single directory (no granular control)
- Client cannot delete or update files on server (No complete CRUD operations)
- When client receives a file's content it just verifies it and quits. Doesn't store it on disk.
- Files are sent one-by-one should implement batch sending

# Future work
- Add user authentication (at least a password in the request)
- Send files encrypted (e.g. https)
- Support CRUD operations (maybe this will require the client storing and updating the merkle tree on disk)
