# Muxis

> High-performance Redis client for Rust with multiplexing and cluster support

**Version**: 0.3.0 (Phase 3 Complete - Standalone API)

[![Rust](https://img.shields.io/badge/rust-1.83%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

## Features

- **Complete RESP2 Protocol** - Full Redis Serialization Protocol support
- **Multiplexed Connections** - Multiple concurrent requests over single connection
- **Type-Safe Commands** - Rust-native API with compile-time safety
- **75+ Redis Commands** - String, Hash, List, Set, Sorted Set operations
- **Cluster Support** - Slot-based routing, topology discovery (feature flag: `cluster`)
- **Connection Management** - Timeouts, authentication, database selection
- **Security** - Password and ACL authentication support
- **Production Ready** - 180 tests (111 core + 69 cluster), zero warnings

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

### Cluster Mode

```rust
use muxis::cluster::ClusterClient;
use bytes::Bytes;

#[tokio::main]
async fn main() -> muxis::Result<()> {
    // Connect to Redis Cluster (seed nodes)
    let client = ClusterClient::connect("127.0.0.1:7000,127.0.0.1:7001").await?;

    // Commands are automatically routed to correct node based on key slot
    client.set("mykey", Bytes::from("Hello, Cluster!")).await?;
    
    if let Some(value) = client.get("mykey").await? {
        println!("Value: {}", String::from_utf8_lossy(&value));
    }

    // Check cluster status
    println!("Cluster nodes: {}", client.node_count());
    println!("Fully covered: {}", client.is_fully_covered());

    Ok(())
}
```

More examples available in the `examples/` directory:
- `basic.rs` - Basic Redis operations
- `builder.rs` - Using ClientBuilder for configuration
- `pipeline.rs` - Multiple commands in sequence
- `auth.rs` - Authentication and database selection

## Supported Commands

**Phase 3 Complete**: 75 Redis commands implemented across 5 categories

### String Commands (7)
`GET`, `SET`, `MGET`, `MSET`, `SETNX`, `SETEX`, `GETDEL`, `APPEND`, `STRLEN`, `INCR`, `DECR`

### Key Commands (8)
`DEL`, `EXISTS`, `TYPE`, `EXPIRE`, `EXPIREAT`, `TTL`, `PERSIST`, `RENAME`, `SCAN`

### Hash Commands (13)
`HSET`, `HGET`, `HMSET`, `HMGET`, `HGETALL`, `HDEL`, `HEXISTS`, `HLEN`, `HKEYS`, `HVALS`, `HINCRBY`, `HINCRBYFLOAT`, `HSETNX`

### List Commands (14)
`LPUSH`, `RPUSH`, `LPOP`, `RPOP`, `LLEN`, `LRANGE`, `LINDEX`, `LSET`, `LREM`, `LTRIM`, `RPOPLPUSH`, `BLPOP`, `BRPOP`, `LPOS`

### Set Commands (13)
`SADD`, `SREM`, `SPOP`, `SMEMBERS`, `SISMEMBER`, `SCARD`, `SRANDMEMBER`, `SDIFF`, `SINTER`, `SUNION`, `SDIFFSTORE`, `SINTERSTORE`, `SUNIONSTORE`

### Sorted Set Commands (20)
`ZADD`, `ZREM`, `ZRANGE`, `ZRANGEBYSCORE`, `ZRANK`, `ZSCORE`, `ZCARD`, `ZCOUNT`, `ZINCRBY`, `ZREVRANGE`, `ZREVRANK`, `ZREMRANGEBYRANK`, `ZREMRANGEBYSCORE`, `ZPOPMIN`, `ZPOPMAX`, `BZPOPMIN`, `BZPOPMAX`, `ZLEXCOUNT`, `ZRANGEBYLEX`, `ZREMRANGEBYLEX`

### Connection Commands
`PING`, `ECHO`, `AUTH`, `SELECT`, `CLIENT SETNAME`

### Cluster Commands (6)
`CLUSTER SLOTS`, `CLUSTER NODES`, `CLUSTER INFO`, `ASKING`, `READONLY`, `READWRITE`

**Note**: Full cluster support available with `cluster` feature flag. Currently supports:
- CRC16 slot calculation with hash tag support `{...}`
- Topology discovery and management via CLUSTER SLOTS/NODES
- Slot-based routing (16384 slots)
- Connection pooling per node
- Basic operations: GET, SET, DEL, EXISTS

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
- **muxis::cluster**: Cluster support with slot-based routing (feature-gated: `cluster`)
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

- **180 total tests**: 111 unit tests (core) + 69 unit tests (cluster)
- **Protocol tests**: Frame encoding/decoding with edge cases
- **Command tests**: All 75 standalone commands with unit and integration tests
- **Cluster tests**: Slot calculation, error parsing, topology management, connection pooling
- **Connection tests**: Builder patterns, multiplexing, command execution
- **100% public API documentation** with runnable examples
- **Zero clippy warnings**: Clean code following Rust best practices

Run tests:

```bash
# Unit tests (180 tests)
cargo test --all-features --lib

# Integration tests (requires Redis at 127.0.0.1:6379)
cargo test --all-features -- --ignored

# Documentation tests
cargo test --doc --all-features

# All tests
cargo test --all-features
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
- Phase 3: Standalone API (75 commands) - COMPLETE
- Phase 4: Cluster routing foundation - IN PROGRESS (85%)
- Phase 5+: See [ROADMAP.md](ROADMAP.md)

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
