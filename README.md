# merkle-rs
A Merkle tree implementation in Rust 🌲🦀! 

- Supports merkle proof generation/verification
- Contains 2 networked binaries (a client and a server)
- Server API supports uploading and requesting files alongside their Merkle Proofs

# How it works

## Merkle Tree

## Merkle Proof Generation Algorithm (server)

## Merkle Proof Verification Algorithm (client)

# How to run
Building both the client and the server

```bash
$ cargo b
```

Running all tests 

```bash
$ cargo t
```

# Server
This will start the server on port 3000 and with the default directory for server files which is `./server_files`.

```bash
$ cargo r --bin server
```

## Available Commands

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

Running the server with default options

```bash
$ cargo r --bin server
```

Running the server with custom options


```bash
$ cargo r --bin server -- --port 8081 --path="/path/to/server/files"
```


# Client

## Available Commands
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


We can run the client with the default options as seen below. Since the client requires a command to function, running it without one will print a message and quit.

```bash
$ client
Welcome to merkle-rs client 🔑🦀!
Please give a valid command
Run with --help to get the list of available commands
```

Assuming that the server is running on `http://localhost:3000` we can upload to it upload all files of `./client_files`, compute and store the merkle root of the files as a binary file in `./merkle.bin` as well as delete the local copies using the following command:

```bash
$ cargo r --bin client upload
```

Or we can use custom options like so:

```bash
$ cargo r --bin client -- --files-path="/path/to/client/files" --merkle-path="/path/to/merkle.bin" --server-address="http://example.com" upload
```

If the above step was succesful we can request a file from the server and verify its integity. Assuming we had a file called `file1.txt` under `client_files` we can request it alongside its proof like so:

```bash
$ client request "file1.txt"
```

# Limitations/Shortcomings
- Files and their content are stored in RAM when constructing the Merkle Tree (impractical for larger files)
- Cannot incrementally send files, its a single operation for all files and then we cannot incrementally add more.
- If a new set of files is sent Merkle Tree should be re-created from scratch
- No user authentication, anyone can request or upload a file to the server (no multiple users support)
- Files are sent in plain-text
- Client can only upload all of its files found under a single directory (no granular control)
- Client cannot delete or update files on server
- When client receives a file's content it just verifies it and quits. Doesn't store it on disk.

# Future work
- Add user authentication (at least a password in the request)
- Send files encrypted (e.g. https)
- Support CRUD operations (maybe this will require the client storing and updating the merkle tree on disk)
