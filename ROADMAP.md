# Roadmap: Muxis - Redis Client Library for Rust

> High-performance Redis client with multiplexing, auto standalone/cluster detection, and full feature coverage.

## Goals

- **Multiplexing**: Multiple concurrent requests over a single connection (or small pool) with async/await
- **Auto-detect**: Automatically detect Standalone vs Cluster mode and choose appropriate routing
- **Redirect handling**: Support MOVED and ASK redirects for cluster resharding/failover
- **Feature completeness**: Implement all Redis commands from core to advanced features
- **Tokio-first**: Built on Tokio for maximum performance

## Non-goals (Initially)

- Redis Enterprise extensions (will add later)
- Third-party module support from day one (tiered approach)
- Sentinel/complex proxy setups immediately (framework prepared)

---

## Design Principles

- **Async-first** with Tokio, trait-based transport abstraction (TCP/TLS/Unix socket)
- **Layered architecture**: Protocol layer (RESP) → Routing layer (standalone/cluster) → API layer (typed commands)
- **Test-driven**: Integration tests with real Redis (Docker) + cluster configurations
- **Incremental evolution**: Each phase has Definition of Done (DoD) and release milestone

---

## Architecture

### Crate Layout

```
muxis/
├── muxis-proto/          # RESP2/RESP3 codec, streaming parser, frame model
├── muxis-core/           # Connection, multiplexing, request tracking, retry, errors, tracing
├── muxis-client/         # Public API: Client, Connection, command builders, typed replies
├── muxis-cluster/        # Slot map, node discovery, routing, MOVED/ASK handling
└── muxis-test/           # Docker harness, cluster harness, golden tests (internal)
```

### Multiplexing Model

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
│  Each request: RequestId + oneshot response channel          │
│  RESP ordered replies: FIFO queue matching                   │
│                                                              │
│  Note: PubSub requires separate connection (push-based)      │
└─────────────────────────────────────────────────────────────┘
```

### Auto-detect Flow

```
Connect to first node
        │
        ▼
   HELLO 3 (RESP3)
   or fallback RESP2
        │
        ▼
   CLUSTER INFO
        │
        ├─── "ERR cluster support disabled" ──> Standalone Mode
        │
        └─── cluster_state:ok ──> Cluster Mode
                    │
                    ▼
              CLUSTER SLOTS
              or CLUSTER SHARDS
                    │
                    ▼
              Build Slot Map
```

### MOVED/ASK Redirect Flow

```
Send Command
     │
     ▼
┌─────────────────┐
│ Receive Response│
└────────┬────────┘
         │
         ├─── OK ──────────────────────────────> Return Result
         │
         ├─── MOVED <slot> <host:port> ────────> Update Slot Map
         │                                              │
         │                                              ▼
         │                                        Retry Command
         │
         └─── ASK <slot> <host:port> ──────────> Connect to Node
                                                       │
                                                       ▼
                                                 Send ASKING
                                                       │
                                                       ▼
                                                 Retry Command
                                                 (no map update)

Retry Policy:
- Max redirects: 5 (configurable)
- Backoff for network errors
- Auto-retry only for safe commands (GET, MGET, ...) by default
```

---

## Phasing & Milestones

### Phase 0 — Repository Scaffolding & Standards (M0) ✓ COMPLETE

**Goal**: Establish development infrastructure

- [x] Workspace setup with all crates
- [x] CI/CD: fmt, clippy, tests, doc tests, coverage
- [x] MSRV policy: Rust 1.83+
- [x] Feature flags structure:
  - `tls` - TLS/SSL support
  - `resp3` - RESP3 protocol
  - `cluster` - Cluster mode
  - `json` - RedisJSON commands
  - `streams` - Streams commands
  - `tracing` - Observability
- [x] Error model design:
  ```rust
  pub struct RedisError {
      kind: ErrorKind,
      message: String,
      source: Option<Box<dyn Error>>,
  }

  pub enum ErrorKind {
      Io,
      Protocol,
      Auth,
      Moved { slot: u16, addr: String },
      Ask { slot: u16, addr: String },
      ClusterDown,
      // ...
  }
  ```
- [x] Tracing integration setup

**DoD**: `cargo test` passes, publishable skeleton crates ✓

---

### Phase 1 — RESP Codec + Basic Connection (M1) ✓ COMPLETE

**Goal**: Communicate with Redis standalone

#### RESP Protocol Implementation

- [x] RESP2 types:
  - Simple String: `+OK\r\n`
  - Error: `-ERR message\r\n`
  - Integer: `:1000\r\n`
  - Bulk String: `$6\r\nfoobar\r\n`
  - Array: `*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n`
  - Null: `$-1\r\n` or `*-1\r\n`
- [x] Streaming parser (bytes → frames)
- [x] Encoder (commands → bytes)
- [x] Zero-copy where possible (bytes::Bytes)

#### Basic Commands

- [x] `PING` / `PONG`
- [x] `ECHO`
- [x] `GET` / `SET`
- [x] `DEL`
- [x] `INCR` / `DECR`

#### Connection Management

- [x] `AUTH` (password only, ACL username+password)
- [x] `SELECT` (database selection)
- [x] `CLIENT SETNAME`
- [x] Connect timeout
- [x] Read/Write timeout
- [x] Cancellation safety (tokio::select! safe)

**DoD**: Integration tests with `redis:latest` Docker pass ✓
**Note**: 60 tests total (53 unit + 7 doc tests), 100% public API documentation

---

### Phase 2 — Multiplexing Stable (M2)

**Goal**: Single connection handles many concurrent requests

#### Core Implementation

- [ ] `MultiplexedConnection` struct
- [ ] Writer task (serialize commands, write to socket)
- [ ] Reader task (read frames, dispatch to waiters)
- [ ] Request tracking with `RequestId`
- [ ] Oneshot channels for response delivery

#### Flow Control

- [ ] Bounded request queue with backpressure
- [ ] Configurable queue size
- [ ] Timeout per request

#### Reliability

- [ ] Graceful shutdown
- [ ] Reconnect on connection loss (optional, configurable)
- [ ] Connection health check (periodic PING)

#### Observability

- [ ] Metrics hooks:
  - Pending requests count
  - Request latency histogram
  - Connection state
- [ ] Tracing spans per request

**DoD**: Stress test with 10k concurrent GET/SET tasks - no deadlock, no memory leak

---

### Phase 3 — Standalone API Completeness (M3)

**Goal**: Cover everyday Redis commands

#### String Commands

- [ ] `APPEND`, `GETRANGE`, `SETRANGE`
- [ ] `MGET`, `MSET`, `MSETNX`
- [ ] `SETNX`, `SETEX`, `PSETEX`
- [ ] `GETSET` (deprecated) / `GETDEL` / `GETEX`
- [ ] `INCR`, `INCRBY`, `INCRBYFLOAT`
- [ ] `DECR`, `DECRBY`
- [ ] `STRLEN`

#### Key Commands

- [ ] `EXISTS`, `TYPE`, `RENAME`, `RENAMENX`
- [ ] `EXPIRE`, `EXPIREAT`, `PEXPIRE`, `PEXPIREAT`
- [ ] `TTL`, `PTTL`, `EXPIRETIME`, `PEXPIRETIME`
- [ ] `PERSIST`
- [ ] `KEYS` (use with caution)
- [ ] `SCAN`, `HSCAN`, `SSCAN`, `ZSCAN`
- [ ] `UNLINK` (async delete)
- [ ] `DUMP`, `RESTORE`
- [ ] `OBJECT ENCODING/FREQ/IDLETIME`
- [ ] `TOUCH`, `COPY`

#### Hash Commands

- [ ] `HSET`, `HGET`, `HMSET`, `HMGET`
- [ ] `HDEL`, `HEXISTS`
- [ ] `HGETALL`, `HKEYS`, `HVALS`, `HLEN`
- [ ] `HINCRBY`, `HINCRBYFLOAT`
- [ ] `HSETNX`
- [ ] `HRANDFIELD`
- [ ] `HSCAN`

#### List Commands

- [ ] `LPUSH`, `RPUSH`, `LPUSHX`, `RPUSHX`
- [ ] `LPOP`, `RPOP`, `LMPOP`
- [ ] `LRANGE`, `LINDEX`, `LSET`
- [ ] `LLEN`, `LREM`, `LTRIM`
- [ ] `LINSERT`, `LPOS`
- [ ] `BLPOP`, `BRPOP`, `BLMPOP` (blocking)
- [ ] `LMOVE`, `BLMOVE`

#### Set Commands

- [ ] `SADD`, `SREM`, `SPOP`
- [ ] `SMEMBERS`, `SISMEMBER`, `SMISMEMBER`
- [ ] `SCARD`, `SRANDMEMBER`
- [ ] `SDIFF`, `SINTER`, `SUNION`
- [ ] `SDIFFSTORE`, `SINTERSTORE`, `SUNIONSTORE`
- [ ] `SMOVE`
- [ ] `SSCAN`

#### Sorted Set Commands

- [ ] `ZADD`, `ZREM`, `ZPOPMIN`, `ZPOPMAX`
- [ ] `ZRANGE`, `ZRANGEBYSCORE`, `ZRANGEBYLEX`
- [ ] `ZREVRANGE`, `ZREVRANGEBYSCORE`
- [ ] `ZRANK`, `ZREVRANK`, `ZSCORE`, `ZMSCORE`
- [ ] `ZCARD`, `ZCOUNT`, `ZLEXCOUNT`
- [ ] `ZINCRBY`
- [ ] `ZINTER`, `ZUNION`, `ZDIFF` (+ STORE variants)
- [ ] `ZRANDMEMBER`
- [ ] `BZPOPMIN`, `BZPOPMAX`, `BZMPOP`
- [ ] `ZSCAN`

#### Response Types

- [ ] Typed responses: `i64`, `bool`, `String`, `Vec<u8>`, `Vec<T>`, `HashMap<K,V>`
- [ ] Raw frame escape hatch for advanced use
- [ ] Null handling (Option<T>)

**DoD**: Command coverage documentation, tests for each command group

---

### Phase 4 — Auto-detect + Cluster Routing (M4)

**Goal**: Single `Client::connect(url)` works for both standalone and cluster

#### Mode Detection

- [ ] Detect via `CLUSTER INFO`
- [ ] Fallback detection via `INFO cluster`
- [ ] Cache mode in Client state

#### Slot Map

- [ ] Parse `CLUSTER SLOTS` response
- [ ] Parse `CLUSTER SHARDS` (Redis 7+)
- [ ] Slot → Node mapping (16384 slots)
- [ ] Key hash calculation (CRC16)
- [ ] Hash tag support `{...}`

#### Connection Management

- [ ] Connection pool per node
- [ ] Lazy connection establishment
- [ ] Multiplexed connections to each node

#### Routing

- [ ] Route single-key commands
- [ ] Multi-key command validation:
  - Same slot: allow
  - Different slots: return clear error
- [ ] Helper: `group_by_slot()` for user pipelines
- [ ] Keyless commands → random node

**DoD**: Integration tests with Redis Cluster (3 masters) pass

---

### Phase 5 — Redirect Handling + Topology Refresh (M5)

**Goal**: Cluster resilience during resharding/failover

#### MOVED Handling

- [ ] Parse MOVED error
- [ ] Retry to correct node
- [ ] Update slot map entry
- [ ] Limit redirect count (default: 5)

#### ASK Handling

- [ ] Parse ASK error
- [ ] Send `ASKING` to target node
- [ ] Retry command
- [ ] Do NOT update slot map (migration in progress)

#### Topology Refresh

- [ ] Refresh on MOVED storm (threshold configurable)
- [ ] Periodic background refresh (optional)
- [ ] Refresh on connection failure
- [ ] Atomic slot map swap

#### Node Failure Handling

- [ ] Detect connection failure
- [ ] Mark node as down
- [ ] Trigger topology refresh
- [ ] Retry to new primary

**DoD**: Tests pass for:
- Slot migration during operation
- Primary failover scenario
- Node restart scenario

---

### Phase 6 — Pipelining, Batching & Ergonomics (M6)

**Goal**: Performance optimization and developer experience

#### Pipelining

- [ ] `Pipeline` builder
- [ ] Ordered response collection
- [ ] Atomic send (single write)
- [ ] Works with multiplexing

```rust
let results = client.pipeline()
    .set("key1", "value1")
    .get("key2")
    .incr("counter")
    .execute()
    .await?;
```

#### Auto-batching (Optional)

- [ ] Configurable batch window (e.g., 1ms)
- [ ] Batch small writes together
- [ ] Disable by default

#### Command Builder

- [ ] Low-level `Cmd` builder:
```rust
let cmd = Cmd::new("SET")
    .arg("key")
    .arg("value")
    .arg("EX")
    .arg(60);
```
- [ ] High-level typed methods
- [ ] Method chaining

#### URL Parsing

- [ ] `redis://host:port/db`
- [ ] `rediss://` (TLS)
- [ ] `redis://user:pass@host:port`
- [ ] Query parameters:
  - `?timeout=5s`
  - `?pool_size=10`
  - `?cluster=true`

#### Connection Pool

- [ ] Configurable pool size
- [ ] Connection health check
- [ ] Pool metrics

**DoD**: Benchmarks documented, usage examples complete

---

### Phase 7 — Pub/Sub (M7)

**Goal**: Real-time messaging support

#### Architecture

- [ ] Separate `PubSubConnection` (not multiplexed)
- [ ] Dedicated connection for subscriptions
- [ ] Message stream (tokio channel or async stream)

#### Commands

- [ ] `SUBSCRIBE`, `UNSUBSCRIBE`
- [ ] `PSUBSCRIBE`, `PUNSUBSCRIBE` (pattern)
- [ ] `PUBLISH`
- [ ] `PUBSUB CHANNELS/NUMSUB/NUMPAT`
- [ ] `SSUBSCRIBE`, `SUNSUBSCRIBE` (sharded, Redis 7+)

#### Message Types

```rust
pub enum PubSubMessage {
    Message { channel: String, payload: Bytes },
    PMessage { pattern: String, channel: String, payload: Bytes },
    Subscribe { channel: String, count: i64 },
    Unsubscribe { channel: String, count: i64 },
}
```

#### Reliability

- [ ] Reconnect strategy (optional)
- [ ] Resubscribe on reconnect
- [ ] Connection loss notification

**DoD**: Subscribe/publish integration tests pass

---

### Phase 8 — Transactions, Lua, Streams (M8)

**Goal**: Advanced Redis features

#### Transactions

- [ ] `MULTI` / `EXEC` / `DISCARD`
- [ ] `WATCH` / `UNWATCH`
- [ ] Atomic execution guarantee
- [ ] Cluster: transactions on same slot only

```rust
let results = client.transaction()
    .watch(&["key1", "key2"])
    .multi()
    .get("key1")
    .incr("key2")
    .exec()
    .await?;
```

#### Lua Scripting

- [ ] `EVAL` / `EVALSHA`
- [ ] `SCRIPT LOAD/EXISTS/FLUSH`
- [ ] Script caching (local SHA cache)
- [ ] Cluster: scripts with same-slot keys

```rust
let script = Script::new("return redis.call('get', KEYS[1])");
let result: String = script.key("mykey").invoke(&client).await?;
```

#### Redis Functions (Redis 7+)

- [ ] `FUNCTION LOAD/DELETE/LIST`
- [ ] `FCALL` / `FCALL_RO`

#### Streams

- [ ] `XADD`, `XLEN`, `XRANGE`, `XREVRANGE`
- [ ] `XREAD`, `XREADGROUP`
- [ ] `XGROUP CREATE/DESTROY/SETID`
- [ ] `XACK`, `XCLAIM`, `XAUTOCLAIM`
- [ ] `XPENDING`, `XTRIM`
- [ ] Consumer group management

#### Blocking Commands Caveat

- [ ] Document: blocking ops should use dedicated connection
- [ ] Option: auto-spawn dedicated connection for blocking ops
- [ ] Timeout handling for blocking commands

**DoD**: Transaction/script/stream tests pass, blocking commands documented

---

### Phase 9 — RESP3, Client Caching, Security (M9)

**Goal**: Modern protocol and security features

#### RESP3 Protocol

- [ ] `HELLO 3` handshake
- [ ] New types:
  - Map: `%2\r\n...`
  - Set: `~3\r\n...`
  - Null: `_\r\n`
  - Boolean: `#t\r\n` / `#f\r\n`
  - Double: `,3.14\r\n`
  - Big number: `(3492890328409238509324850943850943825024385\r\n`
  - Verbatim string: `=15\r\ntxt:Some string\r\n`
  - Push: `>3\r\n...`
- [ ] Attribute handling
- [ ] Push message handling

#### Client-Side Caching

- [ ] `CLIENT TRACKING ON`
- [ ] Invalidation message handling
- [ ] Local cache implementation (optional)
- [ ] Cache statistics

#### Security

- [ ] ACL support: `AUTH username password`
- [ ] TLS configuration:
  - Certificate validation
  - Client certificates
  - Custom CA
- [ ] Secure connection options

**DoD**: RESP3 compatibility tests, TLS tests, Redis version matrix

---

### Phase 10 — Sentinel + Replica Routing (M10)

**Goal**: Production topology options

#### Sentinel Support

- [ ] Discover master from Sentinel
- [ ] `SENTINEL GET-MASTER-ADDR-BY-NAME`
- [ ] Auto-failover detection
- [ ] Sentinel connection pool
- [ ] Multi-sentinel configuration

```rust
let client = Client::sentinel()
    .master_name("mymaster")
    .sentinel("sentinel1:26379")
    .sentinel("sentinel2:26379")
    .sentinel("sentinel3:26379")
    .connect()
    .await?;
```

#### Cluster Replica Routing

- [ ] `READONLY` mode
- [ ] Read from replicas option
- [ ] Routing policies:
  - `Primary` - always primary
  - `PreferReplica` - replica if available
  - `RoundRobin` - distribute reads
  - `Nearest` - lowest latency (if measurable)

#### Connection Affinity

- [ ] Sticky connections for specific use cases
- [ ] Connection tagging

**DoD**: Sentinel integration tests, replica routing tests

---

### Phase 11 — Full Feature Coverage (Ongoing)

#### Tier A (Must Have) — Core Commands

| Category | Status | Commands |
|----------|--------|----------|
| Strings | | All basic string operations |
| Keys | | Expiry, scan, type management |
| Hashes | | All hash operations |
| Lists | | All list operations including blocking |
| Sets | | All set operations |
| Sorted Sets | | All zset operations |
| Pub/Sub | | Subscribe, publish, patterns |
| Transactions | | MULTI/EXEC/WATCH |
| Scripting | | EVAL, Functions |
| Streams | | Full stream support |
| Cluster | | Slots, nodes, routing |

#### Tier B (Nice to Have) — Extended Features

| Category | Status | Commands |
|----------|--------|----------|
| HyperLogLog | | PFADD, PFCOUNT, PFMERGE |
| Bitmap | | SETBIT, GETBIT, BITOP, BITCOUNT, BITPOS, BITFIELD |
| Geo | | GEOADD, GEODIST, GEOHASH, GEOPOS, GEORADIUS, GEOSEARCH |
| Server | | INFO, CONFIG, DEBUG, SLOWLOG, MEMORY |
| Connection | | CLIENT, SELECT, QUIT, RESET |

#### Tier C (Redis Stack / Modules)

| Module | Status | Feature Flag |
|--------|--------|--------------|
| RedisJSON | | `json` |
| RediSearch | | `search` |
| RedisTimeSeries | | `timeseries` |
| RedisBloom | | `bloom` |
| RedisGraph | | `graph` (deprecated) |

#### Tier D (Edge Cases)

- [ ] Generic module command support
- [ ] Custom command registration
- [ ] RESP3 push notifications
- [ ] Client-side caching advanced features

**DoD**: Command tracking table (CSV/Markdown) with: supported / partial / missing / test status

---

## Testing Strategy

### Unit Tests

- [ ] RESP parser with fuzzing (cargo-fuzz)
- [ ] Golden test vectors for protocol
- [ ] Error parsing (MOVED, ASK, etc.)
- [ ] Slot calculation
- [ ] URL parsing

### Integration Tests

- [ ] Docker standalone (`redis:latest`, `redis:7`, `redis:6`)
- [ ] Docker cluster (3 masters, 3 replicas)
- [ ] TLS configuration
- [ ] ACL/Auth scenarios

### Scenario Tests

- [ ] Cluster resharding during operations
- [ ] Primary failover
- [ ] Node restart
- [ ] Network partition (simulated)

### Load Tests

- [ ] Concurrent multiplexing (10k+ tasks)
- [ ] Pipeline throughput
- [ ] PubSub message rate
- [ ] Memory usage under load

### Compatibility Matrix

| Redis Version | RESP2 | RESP3 | Cluster | Streams | Functions |
|---------------|-------|-------|---------|---------|-----------|
| 6.0 | | | | | |
| 6.2 | | | | | |
| 7.0 | | | | | |
| 7.2 | | | | | |
| 7.4 | | | | | |

---

## Release Strategy

### Pre-1.0 Releases

| Version | Milestone | Features |
|---------|-----------|----------|
| 0.1.0 | M0 + M1 | RESP codec, basic connection |
| 0.2.0 | M2 | Multiplexing stable |
| 0.3.0 | M3 | Standalone command coverage |
| 0.4.0 | M4 + M5 | Cluster support with redirects |
| 0.5.0 | M6 | Pipelining, ergonomics |
| 0.6.0 | M7 | Pub/Sub |
| 0.7.0 | M8 | Transactions, Lua, Streams |
| 0.8.0 | M9 | RESP3, TLS, Security |
| 0.9.0 | M10 | Sentinel, replica routing |

### 1.0 Criteria

- [ ] API stabilized (no breaking changes planned)
- [ ] Tier A command coverage complete
- [ ] Cluster redirects battle-tested
- [ ] Comprehensive documentation
- [ ] Performance benchmarks published
- [ ] Production usage feedback incorporated

### Post-1.0

- Semantic versioning
- Deprecation policy
- LTS considerations

---

## Benchmarks

### Metrics to Track

- Operations per second (single connection)
- Operations per second (multiplexed)
- P50/P95/P99 latency
- Memory usage
- Connection establishment time

### Comparison Targets

- `redis-rs` (current popular choice)
- `fred` (another Rust client)
- `redis-py` + `hiredis`
- `ioredis` (Node.js)
- Direct `redis-benchmark` baseline

---

## Documentation

### API Documentation

- [ ] Rustdoc for all public APIs
- [ ] Examples in doc comments
- [ ] Feature flag documentation

### Guides

- [ ] Getting Started
- [ ] Connection Configuration
- [ ] Cluster Usage
- [ ] Pub/Sub Patterns
- [ ] Transaction Patterns
- [ ] Error Handling
- [ ] Performance Tuning
- [ ] Migration from redis-rs

### Examples

- [ ] Basic usage
- [ ] Connection pooling
- [ ] Cluster operations
- [ ] Pub/Sub chat
- [ ] Stream processing
- [ ] Rate limiting
- [ ] Caching patterns

---

## Open Questions

1. **Connection string format**: Use URL-based (`redis://`) or builder pattern or both?
   - Recommendation: Both, URL for simple cases, builder for complex

2. **Default pool size**: What's a sensible default?
   - Recommendation: 1 multiplexed connection for most cases

3. **Retry policy**: Exponential backoff parameters?
   - Recommendation: Start 10ms, max 1s, jitter enabled

4. **RESP3 default**: Should RESP3 be default when Redis supports it?
   - Recommendation: Yes, with fallback to RESP2

5. **Blocking command handling**: Dedicated connection or user responsibility?
   - Recommendation: Document clearly, provide helper for dedicated connection

---

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for development setup and contribution guidelines.

## License

MIT OR Apache-2.0 (dual license, common for Rust ecosystem)
