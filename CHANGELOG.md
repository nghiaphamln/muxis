# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
