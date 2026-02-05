# AGENTS.md - Muxis Development Guidelines

> Guidelines for AI agents and contributors working on the Muxis Redis client library.

## Project Overview

Muxis is a high-performance Redis client for Rust with multiplexing, auto standalone/cluster detection, and full feature coverage. Built on Tokio for maximum performance.

## Build Commands

```bash
# Build all crates
cargo build --all-targets --all-features

# Build release
cargo build --release --all-targets --all-features

# Check without building (faster)
cargo check --all-targets --all-features
```

## Code Quality (MANDATORY before each commit)

```bash
# Format check
cargo fmt --all -- --check

# Format fix
cargo fmt --all

# Clippy - MUST pass with no warnings
cargo clippy --all-targets --all-features -- -D warnings

# Full quality check (run after EVERY feature)
cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings
```

**CRITICAL**: Never use `#[allow(...)]` to bypass clippy warnings. Fix the root cause.

## Test Commands

```bash
# Run all tests
cargo test --all-targets --all-features

# Run single test by name
cargo test test_name --all-features

# Run tests in specific crate
cargo test -p muxis-proto --all-features

# Run tests matching pattern
cargo test resp_parser --all-features

# Run with output
cargo test --all-features -- --nocapture

# Run ignored tests (integration)
cargo test --all-features -- --ignored

# Doc tests only
cargo test --doc --all-features
```

## Documentation

```bash
# Build docs
cargo doc --all-features --no-deps

# Build and open docs
cargo doc --all-features --no-deps --open

# Check doc coverage
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps
```

## Directory Structure

```
muxis/
├── Cargo.toml              # Workspace manifest
├── muxis-proto/            # RESP codec crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # Public exports
│       ├── frame/          # Frame types
│       │   ├── mod.rs
│       │   ├── types.rs
│       │   └── value.rs
│       ├── codec/          # Encoder/decoder
│       │   ├── mod.rs
│       │   ├── encoder.rs
│       │   └── decoder.rs
│       └── error.rs
├── muxis-core/             # Core connection crate
├── muxis-client/           # Public API crate
├── muxis-cluster/          # Cluster support crate
└── tests/                  # Integration tests
    ├── standalone.rs
    └── cluster.rs
```

**Rules**:
- Each module = one directory
- Each feature = separate file within module directory
- `mod.rs` only re-exports, no implementation
- Keep files under 500 lines, split if larger

## Code Style

### Language
- **English only** in code, comments, docs, and commit messages
- **No non-ASCII characters** anywhere in source files
- **No emojis** in code or documentation

### Formatting
- Use `rustfmt` defaults (no custom rustfmt.toml unless necessary)
- Max line length: 100 characters
- Use trailing commas in multi-line constructs

### Imports
```rust
// Order: std -> external -> crate -> super -> self
use std::collections::HashMap;
use std::io::{self, Read, Write};

use bytes::{Buf, BufMut, Bytes};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::error::Result;
use super::frame::Frame;
```

### Naming
- Types: `PascalCase` (e.g., `MultiplexedConnection`)
- Functions/methods: `snake_case` (e.g., `parse_frame`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `MAX_FRAME_SIZE`)
- Modules: `snake_case` (e.g., `slot_map`)
- Feature flags: `kebab-case` in Cargo.toml (e.g., `client-side-caching`)

### Types
- Prefer explicit types over inference for public APIs
- Use `Bytes` over `Vec<u8>` for network data
- Use newtypes for domain concepts (e.g., `Slot(u16)`, `NodeId(String)`)

### Error Handling
```rust
// Use thiserror for library errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("connection failed: {0}")]
    Connection(#[from] io::Error),
    
    #[error("protocol error: {message}")]
    Protocol { message: String },
}

// Use Result type alias
pub type Result<T> = std::result::Result<T, Error>;

// Never panic in library code, return Result
// Never use .unwrap() except in tests
```

### Documentation
```rust
/// Short summary (one line).
///
/// Longer description if needed. Explain the "why" not just "what".
///
/// # Arguments
///
/// * `key` - The Redis key to get
///
/// # Returns
///
/// The value if exists, `None` otherwise.
///
/// # Errors
///
/// Returns error if connection fails or protocol error occurs.
///
/// # Examples
///
/// ```rust
/// let value = client.get("mykey").await?;
/// ```
pub async fn get(&self, key: &str) -> Result<Option<Bytes>> {
    // ...
}
```

**Every public item MUST have documentation.**

## Testing Requirements

### Unit Tests
- Place in same file with `#[cfg(test)]` module
- Use descriptive test names: `test_parse_bulk_string_empty`
- Cover happy path, edge cases, and error conditions
- Use `mockall` or similar for mocking dependencies

### Integration Tests
- Place in `tests/` directory
- Require `--ignored` flag (need Docker)
- Test with real Redis instances
- Cover cluster scenarios (resharding, failover)

### Test Coverage
- All public APIs must have tests
- All error paths must have tests
- Edge cases: empty input, max values, unicode (if applicable)

## API Design

- Design for forward compatibility (avoid breaking changes)
- Use builder pattern for complex configurations
- Prefer `impl Into<T>` over concrete types in public APIs
- Use `#[non_exhaustive]` on public enums and structs with fields
- Mark experimental APIs with `#[doc(hidden)]` until stable

## Git Workflow

### Branch Naming
```
feat/short-description     # New feature
fix/issue-description      # Bug fix
refactor/what-changed      # Code refactoring
docs/what-updated          # Documentation
test/what-tested           # Test additions
chore/task-description     # Maintenance tasks
```

### Commit Messages (Conventional Commits)
```
<type>(<scope>): <subject>

[optional body]

[optional footer]
```

**Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `perf`, `ci`

**Examples**:
```
feat(proto): add RESP3 parser support
fix(cluster): handle MOVED redirect correctly
docs(client): add connection pool examples
test(core): add multiplexing stress tests
refactor(proto): split decoder into streaming chunks
```

## Workflow Checklist

Before marking a task complete:
1. [ ] Code compiles: `cargo check --all-targets --all-features`
2. [ ] Format pass: `cargo fmt --all`
3. [ ] Clippy pass: `cargo clippy --all-targets --all-features -- -D warnings`
4. [ ] Tests pass: `cargo test --all-targets --all-features`
5. [ ] Docs build: `cargo doc --all-features --no-deps`
6. [ ] No `#[allow(...)]` added
7. [ ] Public APIs documented
8. [ ] Commit message follows conventional commits
