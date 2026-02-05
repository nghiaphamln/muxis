# Multiplexing

Muxis uses a sophisticated multiplexing architecture to handle multiple concurrent requests efficiently over a single TCP connection.

## Overview

Traditional Redis clients create a new connection for each concurrent request, which adds overhead. Muxis multiplexes multiple requests over one connection using pipelined commands and strict FIFO ordering.

## Architecture

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
└─────────────────────────────────────────────────────────────┘
```

### Components

**Request Queue (mpsc channel)**
- Bounded channel for backpressure control
- Clients send commands via channel
- Writer task pulls and serializes commands

**Writer Task**
- Background Tokio task
- Serializes commands to RESP format
- Writes to TCP socket
- Tracks request IDs in FIFO order

**Reader Task**
- Background Tokio task  
- Reads RESP frames from TCP socket
- Matches responses to pending requests using FIFO order
- Sends responses via oneshot channels

**Pending Map (VecDeque)**
- Maintains FIFO queue of pending requests
- Each entry has oneshot sender for response
- Responses dispatched in order received

## How It Works

### Single Request Flow

```rust
// 1. Client sends command
let response = client.get("mykey").await?;
```

Internal steps:
1. Client creates oneshot channel for response
2. Command sent to Writer Task via mpsc
3. Writer serializes and writes to socket
4. Writer adds oneshot sender to Pending Map
5. Reader reads response frame from socket
6. Reader pops front of Pending Map (FIFO)
7. Reader sends response via oneshot
8. Client receives response

### Concurrent Requests

```rust
use tokio::task;

// Spawn 100 concurrent requests
let handles: Vec<_> = (0..100)
    .map(|i| {
        let client = client.clone(); // Cheap Arc clone
        task::spawn(async move {
            client.set(&format!("key:{}", i), &format!("value:{}", i)).await
        })
    })
    .collect();

// All requests multiplexed over single connection
for handle in handles {
    handle.await??;
}
```

All 100 requests share:
- One TCP connection
- One Writer Task serializing commands
- One Reader Task reading responses
- FIFO order guarantees correct matching

## Benefits

### Performance

- **No connection overhead**: Single TCP connection eliminates handshake costs
- **Reduced context switching**: Background tasks handle I/O
- **Pipeline efficiency**: Commands sent without waiting for responses
- **Zero-copy**: Uses `bytes::Bytes` for efficient buffer management

### Resource Efficiency

- **Memory**: One connection vs. N connections for N concurrent clients
- **File descriptors**: Doesn't exhaust fd limits under high concurrency
- **Network**: Better bandwidth utilization with pipelining

### Simplicity

- **No connection pool**: Clone `Client` cheaply via `Arc`
- **Automatic ordering**: FIFO guarantees correct response matching
- **Backpressure**: Bounded queue prevents memory exhaustion

## Configuration

### Queue Size

Default queue size is 1024 requests:

```rust
const DEFAULT_QUEUE_SIZE: usize = 1024;
```

When queue is full:
- `send_command()` blocks until space available
- Provides automatic backpressure
- Prevents memory exhaustion

### Timeouts

Not currently configurable per-request, but connection-level timeouts are supported via `ClientBuilder`.

## Thread Safety

`Client` can be safely shared across threads:

```rust
use std::sync::Arc;

let client = Arc::new(client);

let handles: Vec<_> = (0..10)
    .map(|i| {
        let client = client.clone();
        std::thread::spawn(move || {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                client.get(&format!("key:{}", i)).await
            })
        })
    })
    .collect();
```

Internally, `MultiplexedConnection` uses `Arc` and channels for safe sharing.

## Pipelining vs Multiplexing

**Pipelining**: Sending multiple commands without waiting for responses (application-level batching)

**Multiplexing**: Multiple concurrent clients sharing one connection (connection-level concurrency)

Muxis provides multiplexing. True pipelining (batched commands with single flush) is planned for Phase 6.

## Limitations

### FIFO Ordering

Responses are matched to requests in FIFO order. This means:

- Response N matches request N
- No out-of-order responses
- Blocking commands can stall the queue

**Example issue**:
```rust
// Request 1: Blocking pop (may wait 10s)
client.blpop(&["queue"], 10).await?;

// Request 2: Fast GET (sent immediately after)
client.get("key").await?;

// Request 2 waits for Request 1 to complete!
```

This is a Redis protocol limitation, not a Muxis limitation.

### Pub/Sub

Pub/Sub requires dedicated connections and is not compatible with multiplexing (Redis switches connection to pub/sub mode).

### Transactions

`MULTI/EXEC` transactions work but all commands in the transaction block other requests until `EXEC`.

## Performance Tips

### Clone Client Liberally

`Client` uses `Arc` internally, so cloning is cheap:

```rust
let client1 = client.clone(); // Cheap
let client2 = client.clone(); // Cheap
```

All clones share the same connection and multiplexer.

### Avoid Blocking Commands

Minimize use of blocking commands (`BLPOP`, `BRPOP`, etc.) as they stall the request queue:

```rust
// Instead of long timeouts
client.blpop(&["queue"], 60).await?; // Blocks for 60s

// Use shorter timeouts and retry
loop {
    if let Some(item) = client.blpop(&["queue"], 1).await? {
        break;
    }
}
```

### Batch When Possible

Use multi-key commands instead of many single-key commands:

```rust
// Good: Single command
client.mget(&["key1", "key2", "key3"]).await?;

// Less efficient: Multiple round-trips
client.get("key1").await?;
client.get("key2").await?;
client.get("key3").await?;
```

## Examples

See `examples/` directory for full examples:

- `basic.rs` - Basic usage with multiplexing
- `pipeline.rs` - Sequential command execution

## Next Steps

- [Architecture](architecture.md) - Deep dive into implementation
- [Performance](performance.md) - Benchmarks and optimization tips
- [Commands](commands.md) - Available Redis commands
