# Architecture

This document provides a deep dive into Muxis internal architecture.

## Module Organization

Muxis is organized as a single-crate library with well-defined modules:

```
muxis/
├── proto/       # RESP protocol codec (8 files)
├── core/        # Connection & multiplexing (6 files)
├── cluster/     # Cluster support (6 files)
└── testing/     # Test utilities (feature-gated)
```

### proto Module

RESP (Redis Serialization Protocol) encoding and decoding.

**Files**:
- `frame.rs` - Frame enum (Simple, Error, Integer, Bulk, Array, Null)
- `encoder.rs` - Frame to bytes encoding
- `decoder.rs` - Bytes to frame decoding (streaming parser)
- `parse.rs` - Helper parsing functions
- `mod.rs` - Module exports

**Key Features**:
- Streaming parser for incremental decoding
- Zero-copy with `bytes::Bytes`
- Buffer overflow protection (512MB limit)
- Safe handling of malformed input

**Frame Types**:
```rust
pub enum Frame {
    Simple(String),          // +OK\r\n
    Error(String),           // -ERR message\r\n
    Integer(i64),            // :1000\r\n
    Bulk(Bytes),             // $6\r\nfoobar\r\n
    Array(Vec<Frame>),       // *2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n
    Null,                    // $-1\r\n
}
```

### core Module

Connection management, multiplexing, and command execution.

**Files**:
- `connection.rs` - TCP connection wrapper
- `multiplexed.rs` - MultiplexedConnection implementation
- `command.rs` - Command builders
- `client.rs` - Client API
- `builder.rs` - ClientBuilder pattern
- `error.rs` - Error types

**MultiplexedConnection** (key component):
```rust
pub struct MultiplexedConnection {
    tx: mpsc::Sender<Request>,  // Send commands
    // Background tasks handle I/O
}

struct Request {
    frame: Frame,
    response_tx: oneshot::Sender<Result<Frame>>,
}
```

Two background tasks:
1. **Writer**: Reads from `mpsc::Receiver`, writes to socket
2. **Reader**: Reads from socket, dispatches to waiting requests

### cluster Module

Redis Cluster support with slot routing and resilience.

**Files**:
- `slot.rs` - CRC16 slot calculation, hash tags
- `topology.rs` - Cluster topology representation
- `pool.rs` - Per-node connection pooling
- `client.rs` - ClusterClient with routing logic
- `commands.rs` - Cluster-specific commands
- `errors.rs` - Cluster error parsing (MOVED/ASK)

**Key Components**:

**Slot Calculation**:
```rust
pub fn key_slot(key: &str) -> u16 {
    // CRC16 of key (or hash tag) mod 16384
}
```

**Topology**:
```rust
pub struct ClusterTopology {
    nodes: HashMap<NodeId, NodeInfo>,
    slot_map: [Option<NodeId>; 16384],
}
```

**Connection Pool**:
```rust
pub struct ConnectionPool {
    pools: RwLock<HashMap<NodeId, Vec<NodeConnection>>>,
    config: PoolConfig,
}
```

**ClusterClient**:
```rust
pub struct ClusterClient {
    seed_nodes: Arc<Vec<String>>,
    topology: Arc<RwLock<ClusterTopology>>,
    pool: Arc<ConnectionPool>,
    storm_tracker: Arc<MovedStormTracker>,
}
```

## Request Lifecycle

### Standalone Client

```
User Code
   │
   ├─> client.get("key")
   │      │
   │      ├─> Create Frame
   │      ├─> Send via mpsc
   │      └─> Await oneshot response
   │
   ├─> Writer Task
   │      ├─> Receive from mpsc
   │      ├─> Encode to RESP
   │      ├─> Write to socket
   │      └─> Add to pending queue
   │
   ├─> Reader Task
   │      ├─> Read from socket
   │      ├─> Decode RESP frame
   │      ├─> Pop from pending queue (FIFO)
   │      └─> Send via oneshot
   │
   └─> Result<Option<Bytes>>
```

### Cluster Client

```
User Code
   │
   ├─> cluster_client.get("key")
   │      │
   │      ├─> Calculate slot: key_slot("key")
   │      ├─> Lookup node: topology.get_master_for_slot(slot)
   │      ├─> Get connection: pool.get_connection(node)
   │      │
   │      ├─> Send command
   │      │
   │      ├─> Handle response:
   │      │   ├─ OK -> Return result
   │      │   ├─ MOVED -> Refresh topology, retry
   │      │   ├─ ASK -> Send ASKING, retry once
   │      │   └─ IO Error -> Mark unhealthy, refresh, retry with backoff
   │      │
   │      └─> Result<Option<Bytes>>
```

## Concurrency Model

### Standalone Client

- **Single connection**: All requests share one TCP connection
- **Background tasks**: Tokio tasks handle I/O
- **Bounded queue**: mpsc channel with backpressure (default: 1024)
- **FIFO ordering**: Responses matched to requests in order

### Cluster Client

- **Per-node connections**: Connection pool for each cluster node
- **Shared state**: `RwLock` for topology, `Arc` for client cloning
- **Health tracking**: Atomic counters, `Mutex` for timestamps
- **Thread-safe**: Clone `ClusterClient` across threads/tasks

## Memory Management

### Zero-Copy Parsing

`bytes::Bytes` provides reference-counted byte buffers:

```rust
// No copying - shares underlying buffer
let original = Bytes::from("hello");
let cloned = original.clone(); // Cheap Arc clone
```

Used extensively:
- Frame encoding/decoding
- Command parameters
- Response values

### Bounded Buffers

All buffers have limits to prevent DoS:

```rust
const MAX_FRAME_SIZE: usize = 512 * 1024 * 1024; // 512MB
const DEFAULT_QUEUE_SIZE: usize = 1024;          // Request queue
```

### Connection Pooling

Cluster mode pools connections per node:

```rust
pub struct PoolConfig {
    pub max_idle_time: Duration,        // 60s
    pub health_check_interval: Duration, // 30s
}
```

Idle connections cleaned up automatically.

## Error Handling

### Error Types

```rust
pub enum Error {
    Io { source: std::io::Error },
    Protocol { message: String },
    Server { message: String },
    Moved { slot: u16, address: String },
    Ask { slot: u16, address: String },
    CrossSlot { keys: Vec<String> },
    // ...
}
```

### Error Propagation

- **No panics in library code**: All errors returned as `Result`
- **Thiserror**: Automatic `std::error::Error` implementation
- **Context preservation**: Errors carry relevant information

## Performance Characteristics

### Algorithmic Complexity

| Operation | Standalone | Cluster |
|-----------|-----------|---------|
| Command | O(1) queue + O(n) serialize | O(1) route + O(1) queue + O(n) serialize |
| Slot lookup | N/A | O(1) hash + O(1) array lookup |
| Topology refresh | N/A | O(n) nodes + O(16384) slot map |
| Connection get | O(1) | O(1) hash lookup |

### Memory Usage

Approximate memory per client:

**Standalone**:
- Client struct: ~100 bytes
- Request queue: 1024 × 100 bytes = 100 KB
- Pending map: 1024 × 50 bytes = 50 KB
- Total: ~150 KB + connection buffers

**Cluster**:
- Client struct: ~200 bytes
- Topology: 16384 × 8 bytes (slot map) + node info = ~150 KB
- Connection pools: N nodes × M connections × 150 KB
- Storm tracker: ~100 bytes
- Total: ~200 KB + N × M × 150 KB

### Throughput

Factors affecting throughput:

1. **Network latency**: Round-trip time dominates for small requests
2. **Command size**: Larger commands take longer to serialize/deserialize
3. **Concurrency**: Multiplexing amortizes latency across many requests
4. **Batching**: Multi-key commands more efficient than many singles

Expected performance (localhost Redis):

- Single request: ~0.1 ms (10,000 ops/sec)
- 100 concurrent: ~0.1 ms each (1,000,000 ops/sec aggregate)
- Cluster: Similar but with routing overhead (~5% slower)

## Testing Strategy

### Unit Tests

Located in each module with `#[cfg(test)]`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_frame_encoding() {
        // ...
    }
}
```

### Integration Tests

Located in `tests/` directory, marked `#[ignore]`:

```rust
#[tokio::test]
#[ignore]
async fn test_cluster_failover() {
    // Requires real Redis Cluster
}
```

### Property Testing

Planned for protocol layer using `proptest` (Phase 6+).

### Benchmarks

Located in `benches/` using Criterion:

```rust
fn bench_set(c: &mut Criterion) {
    c.bench_function("set", |b| {
        b.to_async(rt).iter(|| async {
            client.set("key", "value").await.unwrap()
        })
    });
}
```

## Security Considerations

### Input Validation

- URL parsing with strict scheme validation
- Buffer size limits prevent memory exhaustion
- No unsafe code in protocol layer

### Denial of Service Protection

- Bounded request queues
- Frame size limits (512 MB)
- Connection timeouts
- Idle connection cleanup

### Authentication

- Password via URL or `auth()` command
- ACL username support (Phase 9)
- TLS support via `rediss://` (Phase 9)

## Debugging

### Logging

Muxis uses `tracing` for structured logging:

```rust
use tracing::info;

info!("connection established to {}", addr);
```

Enable logs:
```bash
RUST_LOG=muxis=debug cargo run
```

### Common Issues

**Connection refused**:
- Check Redis is running: `redis-cli ping`
- Check address/port: `redis://127.0.0.1:6379`

**CROSSSLOT errors**:
- Use hash tags: `{user:1}:name` and `{user:1}:age`

**Timeouts**:
- Increase timeout: `ClientBuilder::timeout(Duration::from_secs(30))`

## Future Enhancements

See [ROADMAP.md](../ROADMAP.md) for planned features:

- Phase 6: Pipelining API (batched commands)
- Phase 7: Pub/Sub support
- Phase 8: Transactions (MULTI/EXEC), Lua scripting, Streams
- Phase 9: RESP3, TLS, advanced authentication
- Phase 10: Sentinel support, replica routing
