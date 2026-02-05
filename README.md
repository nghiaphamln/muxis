# Muxis

A high-performance Redis client for Rust with multiplexing, auto standalone/cluster detection, and full feature coverage. Built on Tokio for maximum performance.

## Features

### Current (v0.3.0)

- **Complete RESP2 Protocol**: Full implementation of Redis Serialization Protocol version 2
- **Multiplexed Connections**: Handle multiple concurrent requests over a single TCP connection
- **Async/Await**: Built on Tokio for efficient asynchronous I/O
- **Type-Safe Commands**: Strongly typed command builders and response parsing
- **Connection Management**: Configurable timeouts, graceful shutdown, and connection pooling
- **Security**: URL validation, buffer overflow protection, and DOS prevention
- **Production Ready**: Comprehensive test coverage (60+ tests) and 100% documented public API

### Planned

- Automatic cluster/standalone detection
- MOVED/ASK redirect handling for cluster resharding
- RESP3 protocol support with client-side caching
- Pub/Sub with pattern subscriptions
- Transactions (MULTI/EXEC/WATCH)
- Lua scripting and Redis Functions
- Redis Streams support
- TLS/SSL connections
- Sentinel support
- Complete Redis command coverage

See [ROADMAP.md](ROADMAP.md) for detailed development plan.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
muxis = "0.3"
```

## Quick Start

```rust
use bytes::Bytes;
use muxis::{Client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to Redis
    let mut client = Client::connect("redis://127.0.0.1:6379").await?;

    // Set a value
    client.set("mykey", Bytes::from("Hello, Muxis!")).await?;

    // Get a value
    if let Some(value) = client.get("mykey").await? {
        println!("Value: {}", String::from_utf8_lossy(&value));
    }

    // Increment a counter
    let count = client.incr("counter").await?;
    println!("Counter: {}", count);

    // Delete keys
    client.del("mykey").await?;
    client.del("counter").await?;

    Ok(())
}
```

More examples available in the `examples/` directory:
- `basic.rs` - Basic Redis operations
- `builder.rs` - Using ClientBuilder for configuration
- `pipeline.rs` - Multiple commands in sequence
- `auth.rs` - Authentication and database selection

## Usage Examples

### Connection Configuration

```rust
use muxis::ClientBuilder;

let mut client = ClientBuilder::new()
    .address("redis://127.0.0.1:6379")
    .database(2)
    .build()
    .await?;
```

### Basic Commands

```rust
use bytes::Bytes;

// String operations
client.set("user:1:name", Bytes::from("Alice")).await?;
let name = client.get("user:1:name").await?;

// Atomic operations
let views = client.incr("page:views").await?;
let likes = client.decr("post:dislikes").await?;

// Key deletion
client.del("key1").await?;
client.del("key2").await?;

// Ping server
let pong = client.ping().await?;
```

### Multiplexed Concurrent Requests

```rust
use tokio::task;

// Spawn multiple concurrent requests
let handles: Vec<_> = (0..100)
    .map(|i| {
        let client = client.clone();
        task::spawn(async move {
            client.set(&format!("key:{}", i), &format!("value:{}", i)).await
        })
    })
    .collect();

// Wait for all requests to complete
for handle in handles {
    handle.await??;
}
```

### Error Handling

```rust
use muxis::Error;

match client.get("mykey").await {
    Ok(Some(value)) => println!("Value: {:?}", value),
    Ok(None) => println!("Key not found"),
    Err(e) => println!("Error: {}", e),
}
```

## Architecture

Muxis is organized as a single-crate library with well-defined modules:

### Modules

- **muxis::proto**: RESP protocol codec (encoder/decoder)
- **muxis::core**: Connection management, multiplexing, and command execution
- **muxis::cluster**: Cluster support (planned, feature-gated)
- **muxis::testing**: Test utilities (feature-gated with `test-utils`)

### Multiplexing Model

Muxis uses a sophisticated multiplexing architecture to handle concurrent requests efficiently:

```
┌─────────────────────────────────────────────────────────────┐
│                    MultiplexedConnection                     │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐         ┌──────────────┐                  │
│  │ Request Queue│ ──────> │ Writer Task  │ ────> TCP Socket │
│  │   (mpsc)     │         │              │                  │
│  └──────────────┘         └──────────────┘                  │
│                                                              │
│  ┌──────────────┐         ┌──────────────┐                  │
│  │ Pending Map  │ <────── │ Reader Task  │ <──── TCP Socket │
│  │ (VecDeque)   │         │              │                  │
│  └──────────────┘         └──────────────┘                  │
│                                                              │
│  Each request: Request sent via bounded channel              │
│  RESP ordered replies: Strict FIFO queue matching            │
└─────────────────────────────────────────────────────────────┘
```

**Key Features**:
- Background writer task serializes commands and writes to socket
- Background reader task reads frames and dispatches to waiting requests
- Request ID tracking with oneshot channels for response delivery
- Bounded request queue with backpressure
- FIFO ordering guarantees for pipelined requests

### Protocol Layer

The RESP2 protocol implementation provides:
- Streaming parser for incremental frame decoding
- Zero-copy encoding using `bytes::Bytes`
- Buffer overflow protection (configurable 512MB max)
- Safe handling of malformed input without panics

## Performance

Muxis is designed for high performance:

- **Zero-copy parsing** where possible using `bytes::Bytes`
- **Multiplexing** eliminates connection overhead for concurrent operations
- **Bounded queues** with backpressure prevent memory exhaustion
- **Tokio-native** for efficient async I/O
- **Minimal allocations** in hot paths

## Security

- **URL validation**: Strict parsing and scheme validation (redis:// and rediss://)
- **DOS protection**: Configurable buffer limits prevent memory exhaustion attacks
- **Safe error handling**: No panics in library code, all errors returned as Results
- **Timeout enforcement**: Configurable timeouts for all I/O operations

## Testing

Muxis has comprehensive test coverage:

- **60+ tests**: 53 unit tests + 7 doc tests
- **Protocol tests**: Frame encoding/decoding with edge cases
- **Connection tests**: Builder patterns, multiplexing, command execution
- **100% public API documentation** with runnable examples
- **Integration tests**: Real Redis instances (requires Docker, use `--ignored` flag)

Run tests:

```bash
# Unit tests
cargo test --all-features

# Integration tests (requires Redis)
cargo test --all-features -- --ignored

# Documentation tests
cargo test --doc --all-features
```

## Development

### Requirements

- Rust 1.83 or later (enforced via `rust-toolchain.toml`)
- Docker (for integration tests)

### Building

```bash
# Build all crates
cargo build --all-targets --all-features

# Build release version
cargo build --release --all-targets --all-features

# Check without building (faster)
cargo check --all-targets --all-features
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Full quality check (required before commit)
cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings
```

### Documentation

```bash
# Build documentation
cargo doc --all-features --no-deps

# Build and open in browser
cargo doc --all-features --no-deps --open
```

See [AGENTS.md](AGENTS.md) for detailed development guidelines.

## Project Status

Muxis is in active development. Version 0.3.0 provides a solid foundation with RESP2 protocol support and multiplexed connections for standalone Redis servers.

**v0.3.0 Changes:**
- Refactored from multi-crate workspace to single-crate architecture
- Simplified public API with cleaner imports
- Added comprehensive examples in `examples/` directory
- Improved module organization and documentation

Current status:
- Phase 0: Repository scaffolding - COMPLETE
- Phase 1: RESP codec + basic connection - COMPLETE
- Phase 2: Multiplexing stable - COMPLETE
- Phase 3+: See [ROADMAP.md](ROADMAP.md)

## Minimum Supported Rust Version (MSRV)

Muxis requires Rust 1.83 or later. The MSRV may be updated in minor version releases.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Key areas for contribution:
- Additional Redis commands
- Cluster support implementation
- RESP3 protocol
- Performance optimizations
- Documentation improvements

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Related Projects

- [redis-rs](https://github.com/redis-rs/redis-rs): Popular Redis client for Rust
- [fred](https://github.com/aembke/fred.rs): Another async Redis client
- [Redis Protocol Specification](https://redis.io/docs/reference/protocol-spec/)

## Acknowledgments

Muxis is built on the shoulders of giants:
- [Tokio](https://tokio.rs/): Asynchronous runtime
- [bytes](https://github.com/tokio-rs/bytes): Efficient byte buffer management
- The Rust community for excellent async ecosystem
