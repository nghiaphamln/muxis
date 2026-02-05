# Contributing to Muxis

Thank you for your interest in contributing to Muxis! We welcome contributions from the community to help make this the best Redis client for Rust.

## Getting Started

1.  **Fork the repository** on GitHub.
2.  **Clone your fork** locally.
3.  **Install Rust**: Ensure you have a recent version of Rust installed (1.83+ required).

## Development Workflow

We follow a standard GitHub pull request workflow.

1.  **Create a branch** for your feature or fix:
    ```bash
    git checkout -b feat/your-feature-name
    # or
    git checkout -b fix/your-bug-fix
    ```
2.  **Make your changes**. Please follow the code style guidelines below.
3.  **Run tests** to ensure no regressions:
    ```bash
    cargo test --all-features
    ```
4.  **Format and lint** your code:
    ```bash
    cargo fmt --all
    cargo clippy --all-targets --all-features -- -D warnings
    ```
5.  **Commit your changes** using [Conventional Commits](https://www.conventionalcommits.org/):
    ```bash
    git commit -m "feat(core): add new connection option"
    ```
6.  **Push to your fork** and open a Pull Request.

## Code Style

- **Rustfmt**: We use standard `rustfmt` settings. Run `cargo fmt` before committing.
- **Clippy**: Code must pass `cargo clippy` with no warnings.
- **Documentation**: All public APIs must be documented. Run `cargo doc --open` to verify.
- **No Emojis**: Please avoid using emojis in code comments or documentation files (except for this guide if necessary).

## Project Structure

Muxis is a single-crate project with the following module structure:

- **muxis::proto**: RESP protocol implementation (codec, frames, encoder, decoder)
- **muxis::core**: Core connection handling and multiplexing logic
- **muxis::cluster**: Cluster support with slot-based routing (feature-gated: `cluster`)
- **muxis::testing**: Test utilities (feature-gated: `test-utils`)

### Feature Flags

- `cluster`: Enable Redis Cluster support
- `tls`: Enable TLS/SSL connection support
- `resp3`: Enable RESP3 protocol support (future)
- `json`: Enable JSON serialization support
- `streams`: Enable Redis Streams support (future)
- `test-utils`: Enable testing utilities

## Testing

- **Unit Tests**: Place unit tests in the same file or a `tests` module within the source file.
- **Integration Tests**: Place integration tests in the `tests/` directory. Tests requiring real Redis should be marked with `#[ignore]`.
- **Benchmarks**: Performance benchmarks go in the `benches/` directory using Criterion.

### Running Tests

```bash
# Unit tests (fast, no external dependencies)
cargo test --all-features --lib

# Integration tests (requires Redis)
cargo test --all-features -- --ignored

# Cluster integration tests (requires Redis Cluster)
cargo test --test cluster_integration --features cluster -- --ignored

# Run benchmarks
cargo bench --bench cluster_benchmark --features cluster
```

### Setting Up Redis for Testing

**Standalone Redis** (for core tests):
```bash
docker run -d --name redis -p 6379:6379 redis:latest
```

**Redis Cluster** (for cluster tests):
```bash
docker run -d --name redis-cluster \
  -p 7000-7005:7000-7005 \
  grokzen/redis-cluster:latest
```

## Cluster Development Guidelines

When working on cluster support (`src/cluster/` module):

### Module Organization

- **slot.rs**: CRC16 slot calculation and hash tag parsing
- **errors.rs**: Cluster-specific error types and parsing (MOVED, ASK, CLUSTERDOWN)
- **commands.rs**: Cluster command builders (CLUSTER SLOTS, CLUSTER NODES, etc.)
- **topology.rs**: Cluster topology management and slot mapping
- **pool.rs**: Connection pooling per node
- **client.rs**: High-level ClusterClient API

### Key Design Principles

1. **Hash Tags**: Always support `{...}` hash tags for multi-key operations
2. **Slot Validation**: Use `validate_same_slot()` to prevent CROSSSLOT errors
3. **Redirect Handling**: MOVED and ASK redirects must be handled transparently
4. **Connection Pooling**: Reuse connections per node, track health
5. **Topology Caching**: Cache topology, refresh on MOVED redirects
6. **Error Propagation**: Return proper Error variants for cluster-specific failures

### Adding New Cluster Commands

When adding a new cluster command:

1. **Add to commands.rs**: Create a builder function returning `Cmd`
   ```rust
   pub fn cluster_mycommand() -> Cmd {
       Cmd::new("CLUSTER").arg("MYCOMMAND")
   }
   ```

2. **Add tests**: Include unit tests for command building
   ```rust
   #[test]
   fn test_cluster_mycommand() {
       let cmd = cluster_mycommand();
       let frame = cmd.into_frame();
       // Assert frame structure
   }
   ```

3. **Add to ClusterClient** (if needed): Wrap in high-level API
   ```rust
   pub async fn my_operation(&self) -> Result<T> {
       let cmd = cluster_mycommand();
       let frame = self.execute_with_redirects(cmd.into_frame(), slot).await?;
       // Parse response
   }
   ```

4. **Document**: Add doc comments with examples
5. **Update CHANGELOG.md**: Note the new command

### Testing Cluster Features

- **Unit tests**: Test slot calculation, parsing, validation (no Redis needed)
- **Integration tests**: Mark with `#[ignore]`, document Redis Cluster requirement
- **Benchmarks**: Add to `benches/cluster_benchmark.rs` for performance tracking

### Common Patterns

**Multi-key validation**:
```rust
let keys = vec!["key1", "key2"];
let slot = ClusterClient::validate_same_slot(&keys)?;
```

**Hash tags for same-slot keys**:
```rust
// All map to same slot
let keys = vec!["user:{123}:name", "user:{123}:email"];
```

**Handling redirects** (automatic in ClusterClient):
```rust
// This automatically handles MOVED/ASK
let value = client.get("mykey").await?;
```

## License

By contributing, you agree that your contributions will be licensed under the project's dual MIT/Apache-2.0 license.
