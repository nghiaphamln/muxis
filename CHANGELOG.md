# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-02-05

### Added

#### Multiplexing Stable (Phase 2)

- **Robust Multiplexing**: Actor-based architecture with independent Reader/Writer tasks
- **Deadlock Prevention**: Split I/O design removing Mutex contention
- **Flow Control**: Bounded channels with configurable backpressure (default 1024)
- **Observability**: Tracing instrumentation for all requests
- **Stress Testing**: Verified with 1000+ concurrent requests

## [0.1.0] - 2026-02-05

### Added

#### Infrastructure (Phase 0)

- **Workspace Setup**: Multi-crate workspace structure:
  - `muxis-proto`: RESP protocol codec
  - `muxis-core`: Core connection and multiplexing
  - `muxis-cluster`: Cluster support (stub)
  - `muxis-client`: Public API re-exports
  - `muxis-test`: Test utilities (minimal)

- **CI/CD Pipeline**: GitHub Actions with comprehensive checks:
  - Format verification (`cargo fmt --check`)
  - Linting (`cargo clippy -- -D warnings`)
  - Compilation (`cargo check --all-features`)
  - Unit tests (`cargo test --all-features`)
  - Documentation build (`cargo doc --no-deps`)

- **MSRV Policy**: Rust 1.83+ enforced via `rust-toolchain.toml`

- **Feature Flags Structure**:
  - `tls`: TLS/SSL support (planned)
  - `resp3`: RESP3 protocol support (planned)
  - `cluster`: Cluster mode support (planned)
  - `json`: RedisJSON commands (planned)
  - `streams`: Streams commands (planned)
  - `tracing`: Observability integration

#### RESP Protocol (Phase 1 - muxis-proto)

- **Complete RESP2 Codec**:
  - Frame types: `SimpleString`, `Error`, `Integer`, `BulkString`, `Array`, `Null`
  - Streaming decoder with incremental parsing
  - Zero-copy encoder using `bytes::Bytes`
  - Buffer overflow protection (512MB default maximum)

- **Error Handling**:
  - `Error`: General protocol errors
  - `EncodeError`: Encoding failures
  - `DecodeError`: Parsing failures with detailed context
  - Proper `Display` and `From` implementations for error conversion

- **Memory Safety**:
  - Configurable buffer limits to prevent DOS attacks
  - Safe handling of malformed input without panics
  - Incremental buffer management

#### Connection & Multiplexing (Phase 1 - muxis-core)

- **Connection Layer**:
  - `Connection` struct wrapping `AsyncRead + AsyncWrite`
  - Frame-level read/write operations
  - Timeout support for all I/O operations
  - Graceful connection management

- **Multiplexed Connection**:
  - Concurrent request handling over single TCP connection
  - Background writer task for command serialization
  - Background reader task for response demultiplexing
  - Request ID tracking with oneshot channels
  - Bounded request queue with backpressure
  - Graceful shutdown handling

- **High-Level Client API**:
  - `Client` struct with typed command methods
  - Builder pattern configuration via `ClientBuilder`
  - Connection timeout and I/O timeout settings
  - Client name configuration
  - Database selection support

- **Command Implementation**:
  - `Cmd` struct for building Redis commands
  - Basic commands: `GET`, `SET`, `DEL`, `INCR`, `DECR`
  - Administrative: `PING`, `ECHO`, `AUTH`, `SELECT`
  - Support for command arguments and options
  - Type-safe response conversion

#### Testing & Documentation

- **Comprehensive Test Coverage** (60 tests total):
  - Protocol tests: Frame encoding/decoding (35 tests)
  - Connection tests: Builder, commands, multiplexing (18 tests)
  - Doc tests: All public API examples (7 tests)
  - Edge case coverage: Empty values, null responses, errors

- **Complete API Documentation**:
  - 100% public API coverage with rustdoc
  - Module-level documentation
  - Usage examples in all doc comments
  - Architecture documentation
  - Error handling patterns

### Security

- **Address Parsing**: Fixed potential security issues in URL parsing
  - Use `url` crate for proper validation
  - Validate URL schemes (only `redis://` and `rediss://`)
  - Prevent malformed address attacks

- **DOS Protection**:
  - Configurable maximum frame size (default 512MB)
  - Buffer overflow protection in decoder
  - Bounded request queues to prevent memory exhaustion

### Fixed

- Removed `panic!` from `Decoder::append()` method
- Fixed validation logic in frame decoder
- Corrected all doc test examples (replaced `ignore` with `no_run` or made runnable)
- Resolved dependency version conflicts for tokio (1.40+)
- MSRV compatibility for native-tls (pinned to 0.2.11)

### Changed

- Decoder validation moved from `append()` to `decode()` for safety
- Switched from cargo-native-tls to tokio-tls for feature management
- MSRV updated to Rust 1.83 for latest async features
