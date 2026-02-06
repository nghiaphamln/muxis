# Muxis

High-performance async Redis client for Rust with multiplexing and cluster support.

[![Crates.io](https://img.shields.io/crates/v/muxis.svg)](https://crates.io/crates/muxis)
[![Documentation](https://docs.rs/muxis/badge.svg)](https://docs.rs/muxis)
[![License](https://img.shields.io/crates/l/muxis.svg)](LICENSE)

## Features

- **Async/Await**: Built on Tokio for high-performance async I/O
- **Multiplexing**: Multiple concurrent requests over single connection
- **Cluster Support**: Production-grade Redis Cluster with automatic routing and failover
- **RESP Protocol**: Full RESP2 support with RESP3 coming soon
- **Type Safety**: Strongly-typed API with comprehensive error handling
- **Zero-Copy**: Efficient parsing using `bytes::Bytes`
- **75+ Commands**: String, Hash, List, Set, Sorted Set operations

## Quick Start

Add Muxis to your `Cargo.toml`:

```toml
[dependencies]
muxis = "0.4"
```

Basic usage:

```rust
use muxis::Client;
use bytes::Bytes;

#[tokio::main]
async fn main() -> muxis::Result<()> {
    let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    
    client.set("mykey", Bytes::from("Hello, Muxis!")).await?;
    
    if let Some(value) = client.get("mykey").await? {
        println!("Value: {}", String::from_utf8_lossy(&value));
    }
    
    Ok(())
}
```

Redis Cluster:

```rust
use muxis::cluster::ClusterClient;

let client = ClusterClient::connect("127.0.0.1:7000,127.0.0.1:7001").await?;
client.set("key", Bytes::from("value")).await?;
```

## Documentation

- **[Getting Started](docs/getting-started.md)** - Installation, basic usage, connection URLs
- **[Commands Reference](docs/commands.md)** - Complete command documentation
- **[Cluster Mode](docs/cluster.md)** - Redis Cluster support, topology, failover
- **[Multiplexing](docs/multiplexing.md)** - How concurrent requests work
- **[Architecture](docs/architecture.md)** - Internal design and implementation

## Feature Flags

Enable optional features in `Cargo.toml`:

```toml
[dependencies]
muxis = { version = "0.4", features = ["cluster", "tls"] }
```

Available features:

| Feature | Description |
|---------|-------------|
| `cluster` | Redis Cluster support with slot routing |
| `tls` | TLS/SSL encrypted connections |
| `resp3` | RESP3 protocol support (experimental) |
| `json` | JSON serialization helpers |
| `streams` | Redis Streams support |
| `test-utils` | Testing utilities for integration tests |

## Project Status

**Current Version**: 0.4.0

**Completed Features**:
- RESP2 protocol codec
- Multiplexed connections
- 75+ Redis commands
- Connection pooling
- Redis Cluster with resilience (MOVED/ASK handling, failure detection, automatic retry)
- Comprehensive documentation

**In Development** (Roadmap):
- Pipelining API
- Pub/Sub support
- Transactions (MULTI/EXEC)
- Lua scripting
- RESP3 protocol
- Sentinel support

See [ROADMAP.md](ROADMAP.md) for detailed development plan.

## Examples

The `examples/` directory contains working examples:

```bash
# Basic usage
cargo run --example basic

# Using ClientBuilder
cargo run --example builder

# Pipeline execution
cargo run --example pipeline

# Authentication
cargo run --example auth

# Redis Cluster
cargo run --example cluster --features cluster
```

## Performance

Muxis is designed for high performance:

- **Multiplexing**: Share single connection across many concurrent requests
- **Zero-copy**: Efficient buffer management with `bytes::Bytes`
- **Connection pooling**: Reuse connections in cluster mode
- **Smart routing**: Direct routing to correct cluster node

Benchmarks available in `benches/` directory:

```bash
cargo bench --features cluster
```

## Testing

Run tests:

```bash
# Unit tests (no Redis required)
cargo test --lib --all-features

# Integration tests (requires Redis)
cargo test --test integration -- --ignored

# Cluster tests (requires Redis Cluster)
cargo test --test cluster_integration -- --ignored
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

For AI coding agents, see [AGENTS.md](AGENTS.md) for development guidelines and commands.

## Versioning

Muxis follows [Semantic Versioning 2.0](https://semver.org/).

Current version `0.4.0` indicates:
- Public API is not yet stable (breaking changes may occur)
- Production-ready for internal use
- Approaching 1.0.0 with feature completion

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Resources

- **Documentation**: https://docs.rs/muxis
- **Repository**: https://github.com/nghiaphamln/muxis
- **Crates.io**: https://crates.io/crates/muxis
- **Roadmap**: [ROADMAP.md](ROADMAP.md)
- **Changelog**: [CHANGELOG.md](CHANGELOG.md)

## Acknowledgments

Muxis is inspired by [mini-redis](https://github.com/tokio-rs/mini-redis) and built with:

- [Tokio](https://tokio.rs/) - Async runtime
- [Bytes](https://github.com/tokio-rs/bytes) - Zero-copy buffer management
- [Thiserror](https://github.com/dtolnay/thiserror) - Error handling
