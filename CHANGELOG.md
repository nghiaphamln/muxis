# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Stability Infrastructure**: Added public API snapshot testing infrastructure.
  - Tests are now in place to detect accidental breaking changes to the public API.
  - Added `#[non_exhaustive]` to `Error` enum and `PoolConfig` struct for future extensibility.
  - Added `Debug` trait implementation for `ClusterClient` to improve developer experience.
  - Added `max_frame_size` configuration to `ClientBuilder` (default 512MB) to control memory limits.

### Changed

- **API Visibility Hardening**: Internal modules are now hidden from the public API.
  - `core`, `proto`, and `cluster` modules are now `pub(crate)` instead of `pub`.
  - Users should import types via `muxis::{Client, ClusterClient, Error, ...}`.
  - This change allows internal refactoring without breaking user code.

### Migration Guide

Users who were importing internal types directly should update their imports:

```rust
// Before (internal paths)
use muxis::cluster::ClusterClient;
use muxis::core::Client;

// After (public API)
use muxis::ClusterClient;
use muxis::Client;
```

## [0.5.0] - 2026-02-06

### Changed

- **TLS Provider**: Switched from `native-tls` to `rustls` for a modern, memory-safe TLS implementation.
  - Replaced `native-tls` and `tokio-native-tls` with `rustls` and `tokio-rustls`.
  - Added `webpki-roots` for secure-by-default certificate validation.
- **Dependency Optimization**: Optimized `tokio` features to reduce compile time and binary size.
  - Removed `full` feature.
  - Enabled only required features: `net`, `sync`, `time`, `rt`, `io-util`, `macros`.

### Fixed

- **TLS Connection**: Fixed logic in `connect_inner` that previously ignored the `is_tls` parameter.
- **Documentation**: Added missing documentation for `TlsConnectorInner`.

## [0.4.0] - 2025-02-05

Major release with Redis Cluster resilience features and comprehensive documentation restructuring.

### Added

**Documentation Restructuring**
- New `docs/` directory with organized documentation
- `docs/getting-started.md` - Installation and basic usage guide
- `docs/commands.md` - Complete command reference (75+ commands)
- `docs/cluster.md` - Redis Cluster setup and usage guide
- `docs/multiplexing.md` - Multiplexing architecture explanation
- `docs/architecture.md` - Internal design and implementation details
- Simplified README.md with table of contents

**Phase 5: Cluster Resilience**

Production-grade resilience features for handling node failures, network issues, and topology changes during Redis Cluster operations.

#### Added

**MOVED Storm Detection**
- `MovedStormTracker` struct for detecting rapid MOVED redirects
- Configurable threshold (10 redirects per 1-second window)
- Automatic topology refresh throttling with 500ms cooldown
- Prevents excessive refresh operations during slot migrations
- 6 unit tests for storm detection, window reset, and cooldown logic

**IO Error Retry with Exponential Backoff**
- Automatic retry on connection failures (max 3 retries)
- Exponential backoff delays: 100ms, 200ms, 400ms
- Integration with topology refresh on IO errors
- Marks failed connections as unhealthy in pool
- Warning logs for retry attempts and failures
- 2 unit tests for retry logic and backoff calculation

**Node Failure Handling**
- Automatic detection of connection failures via IO errors
- Integration with connection pool health tracking
- Automatic topology refresh when nodes become unreachable
- Connection pool filters out unhealthy connections
- Seamless failover to new primary after topology refresh

**Enhanced Error Handling**
- `MAX_RETRIES_ON_IO` constant (default: 3)
- `RETRY_DELAY_MS` constant (default: 100ms base)
- `MOVED_STORM_THRESHOLD` constant (default: 10)
- `MOVED_STORM_WINDOW` constant (default: 1 second)
- `REFRESH_COOLDOWN` constant (default: 500ms)

**Documentation**
- Enhanced `execute_with_redirects()` documentation with resilience features
- MovedStormTracker implementation with detailed comments
- Retry policy documented: All commands are retried on IO errors (limitation noted)

#### Changed

- `ClusterClient` now includes `storm_tracker: Arc<MovedStormTracker>` field
- `execute_with_redirects()` rewritten with IO retry loop and storm detection
- `refresh_topology()` now resets storm tracker on successful refresh
- Imports updated to include `AtomicUsize`, `Ordering`, `Duration`, `Instant`, `Mutex`

### Phase 4: Cluster Support (Complete)

Production-ready Redis Cluster support with automatic redirect handling, slot-based routing, topology management, and connection pooling.

### Added

#### Cluster Infrastructure (`cluster` feature flag)

**Slot Calculation & Routing**
- `key_slot()` - CRC16-CCITT based slot calculation (16384 slots)
- `SLOT_COUNT` constant (16383)
- Hash tag support `{...}` for multi-key operations ensuring same-slot routing
- 16 unit tests covering edge cases and Redis compatibility

**Error Handling**
- `Error::Moved` variant for MOVED redirect errors with slot and address
- `Error::Ask` variant for ASK redirect errors with slot and address
- `Error::ClusterDown` for cluster unavailability
- `Error::CrossSlot` for multi-key commands across different slots
- `parse_redis_error()` function for parsing Redis cluster error responses
- 13 unit tests for error parsing

**Cluster Commands**
- `cluster_slots()` - Query cluster slot distribution
- `cluster_nodes()` - Query cluster node information
- `cluster_info()` - Query cluster state
- `asking()` - Enable ASK redirect handling
- `readonly()` - Enable read operations on replicas
- `readwrite()` - Disable read operations on replicas
- 6 unit tests for command builders

**Topology Management**
- `ClusterTopology` struct for storing cluster state
- `NodeInfo` struct with node metadata (ID, address, flags, slots)
- `NodeFlags` struct for node role and state (master, replica, failing, etc.)
- `NodeId` type for unique node identification
- `SlotRange` struct mapping slot ranges to master and replicas
- `from_cluster_slots()` parser for CLUSTER SLOTS responses
- `from_cluster_nodes()` parser for CLUSTER NODES text responses
- `get_master_for_slot()` for routing queries
- `get_replicas_for_slot()` for read operations
- `build_slot_ranges()` for constructing topology from node information
- 24 unit tests covering topology parsing and lookups

**Connection Pool**
- `ConnectionPool` for managing connections per cluster node
- `NodeConnection` wrapper with health tracking and usage statistics
- `PoolConfig` with configurable limits:
  - `max_connections_per_node` (default: 10)
  - `min_idle_per_node` (default: 1)
  - `max_idle_time` (default: 5 minutes)
  - `health_check_interval` (default: 30 seconds)
- Connection reuse logic with health checking
- Automatic cleanup of idle and unhealthy connections
- 3 unit tests for pool management

**Cluster Client**
- `ClusterClient` struct for cluster operations
- Seed node parsing with comma-separated addresses
- Automatic topology discovery via CLUSTER SLOTS
- Slot-based routing to correct nodes
- Connection pool integration
- Basic command methods:
  - `get(key)` - Get string value with automatic redirect handling
  - `set(key, value)` - Set string value with automatic redirect handling
  - `del(key)` - Delete key with automatic redirect handling
  - `exists(key)` - Check key existence with automatic redirect handling
- Management APIs:
  - `node_count()` - Number of cluster nodes
  - `slot_range_count()` - Number of slot ranges
  - `is_fully_covered()` - Verify all slots are assigned
  - `refresh_topology()` - Manual topology refresh
- 10 unit tests for client initialization and utilities

**Redirect Handling**
- `execute_with_redirects()` - Automatic MOVED/ASK redirect retry logic
- MOVED redirect handling:
  - Detects permanent slot migrations
  - Refreshes topology automatically
  - Retries command on correct node
- ASK redirect handling:
  - Detects temporary slot migrations
  - Sends ASKING command before retry
  - Executes on temporary node without topology update
- `get_connection_for_address()` - Connection management for ASK redirects
- Maximum 5 redirects to prevent infinite loops
- Transparent handling in all cluster commands
- 8 unit tests for redirect logic and edge cases

**Multi-Key Validation**
- `validate_same_slot()` - Public API for multi-key validation
- Prevents CROSSSLOT errors before sending commands
- Validates keys map to same slot (required for cluster multi-key ops)
- Hash tag validation support
- 5 unit tests covering single/multiple keys and edge cases

**Examples & Documentation**
- `examples/cluster.rs` - Comprehensive cluster usage demonstration (180 lines)
  - Connecting to cluster with seed nodes
  - Automatic slot-based routing
  - Hash tag usage for same-slot keys
  - CROSSSLOT error prevention
  - Topology discovery and refresh
  - Docker setup instructions
- All public APIs documented with runnable examples
- Enhanced documentation for redirect handling behavior

### Technical Details

- **Module**: `src/cluster/` with 6 files (2,572 lines)
  - `slot.rs` - Slot calculation (293 lines, 16 tests)
  - `errors.rs` - Error parsing (252 lines, 13 tests)
  - `commands.rs` - Command builders (230 lines, 6 tests)
  - `topology.rs` - Topology management (765 lines, 24 tests)
  - `pool.rs` - Connection pooling (366 lines, 3 tests)
  - `client.rs` - Cluster client API (666 lines, 18 tests)
- **Test Coverage**: 77 new tests (all passing)
  - 69 infrastructure tests (slot, errors, commands, topology, pool)
  - 8 redirect and validation tests
- **Code Quality**: Zero clippy warnings with `-D warnings` flag
- **Documentation**: 100% public API coverage with examples
- **Feature Flag**: All cluster code behind `cluster` feature
- **Examples**: 1 comprehensive example with Docker setup

### Improved

- Error types extended with cluster-specific variants
- Feature flag structure includes `cluster` feature
- All cluster commands now use automatic redirect handling
- Topology refresh integrated with MOVED redirects

## [0.3.0] - 2026-02-05

### Phase 3: Standalone API Completeness

This release completes the standalone Redis API with 75 commands across all major data structures.

### Added

#### String Commands (7 commands)
- `MGET` - Get multiple values
- `MSET` - Set multiple key-value pairs
- `SETNX` - Set if not exists
- `SETEX` - Set with expiration
- `GETDEL` - Get and delete
- `APPEND` - Append to string
- `STRLEN` - Get string length

#### Key Commands (8 commands)
- `EXISTS` - Check if key exists
- `TYPE` - Get key type
- `EXPIRE` - Set expiration in seconds
- `EXPIREAT` - Set expiration at timestamp
- `TTL` - Get time to live
- `PERSIST` - Remove expiration
- `RENAME` - Rename key
- `SCAN` - Iterate keys

#### Hash Commands (13 commands)
- `HSET` - Set hash field
- `HGET` - Get hash field
- `HMSET` - Set multiple hash fields
- `HMGET` - Get multiple hash fields
- `HGETALL` - Get all hash fields and values
- `HDEL` - Delete hash fields
- `HEXISTS` - Check if hash field exists
- `HLEN` - Get hash length
- `HKEYS` - Get all hash keys
- `HVALS` - Get all hash values
- `HINCRBY` - Increment hash field by integer
- `HINCRBYFLOAT` - Increment hash field by float
- `HSETNX` - Set hash field if not exists

#### List Commands (14 commands)
- `LPUSH` - Push to list head
- `RPUSH` - Push to list tail
- `LPOP` - Pop from list head
- `RPOP` - Pop from list tail
- `LLEN` - Get list length
- `LRANGE` - Get range of elements
- `LINDEX` - Get element by index
- `LSET` - Set element by index
- `LREM` - Remove elements
- `LTRIM` - Trim list
- `RPOPLPUSH` - Pop from one list, push to another
- `BLPOP` - Blocking pop from list head
- `BRPOP` - Blocking pop from list tail
- `LPOS` - Find position of element

#### Set Commands (13 commands)
- `SADD` - Add members to set
- `SREM` - Remove members from set
- `SPOP` - Pop random member
- `SMEMBERS` - Get all members
- `SISMEMBER` - Check if member exists
- `SCARD` - Get set cardinality
- `SRANDMEMBER` - Get random member(s)
- `SDIFF` - Set difference
- `SINTER` - Set intersection
- `SUNION` - Set union
- `SDIFFSTORE` - Store set difference
- `SINTERSTORE` - Store set intersection
- `SUNIONSTORE` - Store set union

#### Sorted Set Commands (20 commands)
- `ZADD` - Add members with scores
- `ZREM` - Remove members
- `ZRANGE` - Get members by rank range
- `ZRANGEBYSCORE` - Get members by score range
- `ZRANK` - Get member rank
- `ZSCORE` - Get member score
- `ZCARD` - Get sorted set cardinality
- `ZCOUNT` - Count members in score range
- `ZINCRBY` - Increment member score
- `ZREVRANGE` - Get members in reverse order
- `ZREVRANK` - Get reverse rank
- `ZREMRANGEBYRANK` - Remove by rank range
- `ZREMRANGEBYSCORE` - Remove by score range
- `ZPOPMIN` - Pop member with lowest score
- `ZPOPMAX` - Pop member with highest score
- `BZPOPMIN` - Blocking pop min
- `BZPOPMAX` - Blocking pop max
- `ZLEXCOUNT` - Count by lexicographical range
- `ZRANGEBYLEX` - Get by lexicographical range
- `ZREMRANGEBYLEX` - Remove by lexicographical range

### Improved

- **Test Coverage**: Increased from 60 to 192 total tests
  - 111 unit tests (was 53)
  - 81 integration tests (new)
  - 7 documentation tests
- **Code Quality**: Zero clippy warnings, 100% public API documentation
- **Type Safety**: Added helper functions for parsing optional values and complex responses
  - `frame_to_optional_int()` for nullable integer responses
  - `frame_to_optional_float()` for nullable float responses
  - `frame_to_zpop_result()` for sorted set pop operations
  - `frame_to_bzpop_result()` for blocking sorted set operations

### Technical Details

- All commands implemented in `src/core/command.rs` with builder pattern
- Client methods in `src/core/mod.rs` with full documentation
- Integration tests for each command category in `tests/` directory
- Lifetime issues resolved using `impl Into<Bytes>` pattern for flexible API

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
