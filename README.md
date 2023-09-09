# merkle-rs
A Merkle tree implementation with proof verification and generation alongside a server and client implementation for real world usage. The client can send files to the server and then ask for a given file alongside a Merkle Proof that the file is in the Merkle Tree and has not been tampered with. The client will then verify the proof and choose whether to accept or reject the file.

## How to run
Firstly we need to build the application

```bash
cargo b
```

Running tests 
```bash
cargo t
```

Then we can start the server. This will start the server on port 3000 and with the default directory for server files which is `./server_files`. The server will listen on poret 3000 for 2 types of requests. A POST request for uploading a file and a GET request for asking for a file by name alongside the Merkle proof.

```bash
cargo r --bin server
```

And then the client on another temrinal.

```bash
cargo r --bin client
```

## Limitations
- Files and their content are stored in RAM when constructing the Merkle Tree (impractical for larger files)
- If a new file is sent Merkle Tree should be re-created from scratch
- No user authentication, anyone can request or upload a file to the server
- Files are sent in plain-text
