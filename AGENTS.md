# AGENTS.md - Muxis Development Guidelines

> Concise guidelines for AI agents and contributors working on Muxis Redis client.

## Project Overview

**Muxis** - High-performance Redis client for Rust with multiplexing and cluster support.
- **Version**: 0.3.0 (single-crate architecture)
- **Language**: Rust + Tokio async runtime
- **No publishing to crates.io** (internal project)

## Quick Commands

```bash
# Build
cargo build --all-targets --all-features
cargo check --all-targets --all-features        # Faster, no codegen

# Quality (MANDATORY before commits)
cargo fmt --all                                   # Format code
cargo clippy --all-targets --all-features -- -D warnings  # Lint (must pass)

# Tests
cargo test --all-features                         # All tests
cargo test test_name --all-features              # Single test by name
cargo test proto --all-features                  # Tests in module
cargo test -- --nocapture                        # Show output
cargo test -- --ignored                          # Integration tests (needs Docker)

# Documentation
cargo doc --all-features --no-deps --open        # Build and open docs

# Examples
cargo check --examples --all-features            # Check examples compile
cargo run --example basic                        # Run example (needs Redis)
```

## Project Structure

```
muxis/
├── src/
│   ├── lib.rs              # Public API, re-exports
│   ├── proto/              # RESP protocol codec (8 files)
│   ├── core/               # Connection, multiplexing (6 files)
│   ├── cluster/            # Cluster support (6 files: slot, topology, pool, client, commands, errors)
│   └── testing/            # Test utilities (feature-gated)
├── tests/                  # Integration tests (#[ignore] for cluster tests)
├── benches/                # Criterion benchmarks
└── examples/               # Usage examples (basic, builder, pipeline, auth, cluster)
```

## Code Style

### Language & Formatting
- **English only** - code, comments, docs, commit messages
- **No emojis** - in code or documentation
- **No non-ASCII** - anywhere in source files
- **Max line length**: 100 characters
- Use `rustfmt` defaults (run `cargo fmt --all` before commits)

### Naming Conventions
```rust
// Types: PascalCase
struct MultiplexedConnection { }
enum Frame { }

// Functions/methods: snake_case
async fn parse_frame() { }
pub fn connect_timeout() { }

// Constants: SCREAMING_SNAKE_CASE
const MAX_FRAME_SIZE: usize = 512 * 1024 * 1024;

// Modules: snake_case
mod slot_map;

// Features: kebab-case (in Cargo.toml)
[features]
client-side-caching = []
```

### Import Order
```rust
// 1. std library
use std::collections::HashMap;
use std::io::{self, Read};

// 2. External crates
use bytes::{Buf, BufMut, Bytes};
use tokio::io::AsyncRead;

// 3. Crate internal
use crate::proto::Frame;

// 4. Parent/current module
use super::encoder::Encoder;
```

### Type Conventions
- Prefer explicit types in public APIs
- Use `Bytes` over `Vec<u8>` for network data
- Use newtypes for domain concepts: `Slot(u16)`, `NodeId(String)`
- Use `impl Into<T>` for flexibility in public APIs
- Use `Arc<T>` for shared ownership (especially in async contexts)
- Use `RwLock` for read-heavy shared state, `Mutex` for write-heavy
- Use `AtomicUsize` for counters without lock contention

### Error Handling
```rust
// Use thiserror for library errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("connection failed: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("protocol error: {message}")]
    Protocol { message: String },
}

pub type Result<T> = std::result::Result<T, Error>;

// NEVER panic in library code - always return Result
// NEVER use .unwrap() or .expect() - except in tests
// NEVER use #[allow(...)] to bypass clippy - fix the root cause
```

### Documentation
```rust
/// Short summary (one line).
///
/// Detailed description explaining "why", not just "what".
///
/// # Arguments
///
/// * `key` - Description of key parameter
///
/// # Returns
///
/// Description of return value.
///
/// # Errors
///
/// Returns error if connection fails or protocol error.
///
/// # Examples
///
/// ```no_run
/// # use muxis::{Client, Result};
/// # async fn example() -> Result<()> {
/// let mut client = Client::connect("redis://127.0.0.1").await?;
/// let value = client.get("mykey").await?;
/// # Ok(())
/// # }
/// ```
pub async fn get(&self, key: &str) -> Result<Option<Bytes>> { }
```

**Every public item MUST have documentation** (`#![warn(missing_docs)]` is enforced).

## Git Workflow

### Commit Messages (Conventional Commits)
```
<type>(<scope>): <subject>

[optional body]
```

**Types**: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `perf`

**Examples**:
```
feat(proto): add RESP3 parser support
fix(cluster): handle MOVED redirect correctly
docs: update examples with new API
test(core): add multiplexing stress tests
refactor: simplify connection pooling logic
```

### Pre-Commit Checklist
Before marking work complete:
1. [ ] `cargo check --all-targets --all-features` - compiles
2. [ ] `cargo fmt --all` - formatted
3. [ ] `cargo clippy --all-targets --all-features -- -D warnings` - no warnings
4. [ ] `cargo test --all-features` - all tests pass
5. [ ] `cargo doc --all-features --no-deps` - docs build
6. [ ] No `#[allow(...)]` added
7. [ ] Public APIs documented
8. [ ] Commit message follows convention

## Testing

### Test Types
- **Unit tests**: In same file with `#[cfg(test)]` module
- **Integration tests**: In `tests/` directory with `#[ignore]` attribute (require Docker/Redis)
- **Benchmarks**: In `benches/` directory using Criterion (don't run in CI)
- **Doctests**: In documentation comments (must compile with `--all-features`)

### Test Guidelines
- **Names**: Descriptive with `test_` prefix, e.g., `test_parse_bulk_string_empty`
- **Async tests**: Use `#[tokio::test]` for async functions
- **Time-based tests**: Use `tokio::time::pause()` and `advance()` for deterministic timing
- **Coverage**: All public APIs, error paths, edge cases, boundary conditions
- **Mock external dependencies**: Don't require real Redis for unit tests
- **Integration tests**: Mark with `#[ignore]` if they need external services

### Running Tests
```bash
# All unit tests (fast, no external deps)
cargo test --lib --all-features

# Single test by exact name
cargo test test_storm_tracker_threshold_detection --all-features

# Tests in a module
cargo test cluster::client::tests --all-features

# Integration tests (need Redis/Docker)
cargo test --test cluster_integration -- --ignored

# With output
cargo test test_name -- --nocapture

# Doc tests only
cargo test --doc --all-features
```

## Async Patterns

### Tokio Best Practices
- **Always use `.await`** on async functions (don't block)
- **Don't use `tokio::spawn`** in library code (let users control runtime)
- **Use `tokio::time::sleep`** instead of `std::thread::sleep`
- **Prefer channels** (`mpsc`, `oneshot`) for async communication
- **Use `tokio::select!`** carefully (always handle all branches)

### Concurrency
- **Clone cheap handles**: `Arc<Client>` for multi-threaded access
- **Lock minimally**: Hold locks for shortest duration possible
- **Avoid deadlocks**: Never acquire multiple locks in inconsistent order
- **Use interior mutability**: `Arc<RwLock<T>>` or `Arc<Mutex<T>>` for shared mutable state

## Performance Guidelines

- **Avoid allocations**: Reuse buffers with `Bytes::copy_from_slice` when possible
- **Minimize clones**: Use references or `Arc` instead of cloning large structs
- **Batch operations**: Group Redis commands in pipelines when possible
- **Connection pooling**: Reuse connections, don't create new ones per request
- **Use benchmarks**: Add Criterion benchmarks for performance-critical code

## Cluster-Specific Rules

### Resilience Features (Phase 5 Complete)
- **MOVED storm detection**: Throttle topology refreshes (10 redirects/1s threshold)
- **IO error retry**: Exponential backoff (100ms, 200ms, 400ms), max 3 retries
- **Node failure handling**: Mark unhealthy, refresh topology, route to new master
- **Topology refresh**: Automatic on failures, manual via `refresh_topology()`

### Constants (Don't Change Without Good Reason)
```rust
MAX_REDIRECTS = 5           // MOVED/ASK redirect limit
MAX_RETRIES_ON_IO = 3      // Connection failure retry limit
RETRY_DELAY_MS = 100       // Base delay for exponential backoff
MOVED_STORM_THRESHOLD = 10 // Redirects per window to trigger refresh
MOVED_STORM_WINDOW = 1s    // Time window for storm detection
REFRESH_COOLDOWN = 500ms   // Minimum time between topology refreshes
```

### Testing Cluster Code
- **Unit tests**: Test logic without Redis (use `Arc::new(...)` to construct clients)
- **Integration tests**: Mark with `#[ignore]`, document Docker setup in test comments
- **Storm tracker**: Use `tokio::time::pause()` for deterministic time-based tests
- **Mock connections**: Test retry/failover logic without real network I/O

## Notes for AI Agents

- This is a **single-crate project** (no workspace)
- **Version 0.5.0** with cluster support complete (Phases 4-5)
- Module structure: `proto/` (codec), `core/` (connections), `cluster/` (resilience + routing)
- **Cluster support**: Fully implemented with production-grade resilience (Phase 5 complete)
- Examples: `basic`, `builder`, `pipeline`, `auth`, `cluster`, `cluster_pipeline`
- Feature flags: `tls`, `resp3`, `cluster`, `json`, `streams`, `tracing`, `test-utils`
- **CI setup**: Doesn't run benchmarks (`cargo test --lib` instead of `--all-targets`)
- **No publishing**: Internal project, not published to crates.io
