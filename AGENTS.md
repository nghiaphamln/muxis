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
│   ├── cluster/            # Cluster support (placeholder)
│   └── testing/            # Test utilities (feature-gated)
├── tests/                  # Integration tests
└── examples/               # Usage examples (basic, builder, pipeline, auth)
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

- **Unit tests**: In same file with `#[cfg(test)]`
- **Integration tests**: In `tests/` directory, require `--ignored` flag
- **Test names**: Descriptive, e.g., `test_parse_bulk_string_empty`
- **Coverage**: All public APIs, all error paths, edge cases
- Use `#[tokio::test]` for async tests

## Notes for AI Agents

- This is a **single-crate project** (no workspace)
- **Version 0.3.0** uses simplified public API
- Module structure: `proto/` (codec), `core/` (connections), `cluster/` (future)
- Cluster support is placeholder only (not yet implemented)
- Examples in `examples/` directory demonstrate current API
- Feature flags: `tls`, `resp3`, `cluster`, `json`, `streams`, `tracing`, `test-utils`
