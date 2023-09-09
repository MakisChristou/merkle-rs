# merkle-rs
A Merkle tree implementation that supports proof verification and generation alongside a server and client for real world usage. The client can send files to the server and then ask for a given file alongside a Merkle Proof that the file is in the Merkle Tree and has not been tampered with. The client will then verify the proof and choose whether to accept or reject the file.

## How it works



## How to run
Firstly we need to build the application

```bash
$ cargo b
```

Running tests 

```bash
$ cargo t
```

## Server

Then we can start the server. This will start the server on port 3000 and with the default directory for server files which is `./server_files`. The server will listen on poret 3000 for 2 types of requests. A POST request for uploading a file and a GET request for asking for a file by name alongside the Merkle proof.

```bash
$ cargo r --bin server
```

## Client

### Available Commands

```bash
$ cargo r --bin client -- --help
Welcome to merkle-rs client 🔑🦀!
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


And then the client on another temrinal.

```bash
cargo r --bin client
```

## Limitations/Shortcomings
- Files and their content are stored in RAM when constructing the Merkle Tree (impractical for larger files)
- If a new file is sent Merkle Tree should be re-created from scratch
- No user authentication, anyone can request or upload a file to the server (no multiple users support)
- Files are sent in plain-text
- Client can only upload all of its files found under a single directory (no granular control)
- Client cannot delete or update files on server


## Future work

