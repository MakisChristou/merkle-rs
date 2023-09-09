# merkle-rs


## How to run

Firstly we need to build the application
```bash
cargo b
```

Then we can start the server

```bash
cargo r --bin server
```

And then the client

```bash
cargo r --bin client
```

## Limitations
- Files and their content are stored in RAM (impractical for larger files)
- Merkle Tree should be balanced (?)
- If a new file is sent Merkle Tree should be re-created from scratch
- No user authentication
