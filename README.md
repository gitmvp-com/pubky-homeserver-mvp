# Pubky Homeserver MVP

A minimal viable version of the Pubky homeserver with basic HTTP API and local storage.

## Features

- ✅ HTTP REST API
- ✅ LMDB-based local storage
- ✅ TOML configuration
- ✅ Health check endpoint
- ✅ Simple key-value storage

## What's NOT included (compared to full version)

- ❌ Authentication/Authorization
- ❌ Admin server
- ❌ Pkarr/DHT integration
- ❌ Rate limiting
- ❌ Cloud storage backends
- ❌ WebDAV support
- ❌ TLS/HTTPS

## Installation

```bash
cargo build --release
```

## Usage

### Running the server

```bash
cargo run -- --data-dir ~/.pubky-mvp
```

Or with the release build:

```bash
./target/release/pubky-homeserver-mvp --data-dir ~/.pubky-mvp
```

### Configuration

Copy `config.sample.toml` to your data directory as `config.toml` and customize:

```bash
mkdir -p ~/.pubky-mvp
cp config.sample.toml ~/.pubky-mvp/config.toml
```

## API Endpoints

### Health Check

```bash
curl http://localhost:8080/health
```

Response:
```json
{"status":"ok","version":"0.1.0"}
```

### Store Data

```bash
curl -X PUT http://localhost:8080/data/mykey \
  -H "Content-Type: text/plain" \
  -d "Hello, World!"
```

### Retrieve Data

```bash
curl http://localhost:8080/data/mykey
```

### List All Keys

```bash
curl http://localhost:8080/data
```

### Delete Data

```bash
curl -X DELETE http://localhost:8080/data/mykey
```

## Project Structure

```
pubky-homeserver-mvp/
├── src/
│   ├── main.rs           # Entry point and CLI
│   ├── config.rs         # Configuration types
│   ├── storage.rs        # LMDB storage layer
│   ├── server.rs         # HTTP server and routes
│   └── lib.rs            # Library exports
├── Cargo.toml            # Dependencies
├── config.sample.toml    # Sample configuration
└── README.md             # This file
```

## Development

### Run with debug logging

```bash
RUST_LOG=debug cargo run -- --data-dir ~/.pubky-mvp
```

### Run tests

```bash
cargo test
```

## License

MIT
