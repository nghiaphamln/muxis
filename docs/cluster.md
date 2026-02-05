# Cluster Mode

Muxis provides production-grade Redis Cluster support with automatic slot routing, topology management, and failure resilience.

## Quick Start

```rust
use muxis::cluster::ClusterClient;
use bytes::Bytes;

#[tokio::main]
async fn main() -> muxis::Result<()> {
    // Connect to cluster (provide seed nodes)
    let client = ClusterClient::connect("127.0.0.1:7000,127.0.0.1:7001,127.0.0.1:7002").await?;

    // Commands are automatically routed to correct nodes
    client.set("mykey", Bytes::from("Hello, Cluster!")).await?;
    
    if let Some(value) = client.get("mykey").await? {
        println!("Value: {}", String::from_utf8_lossy(&value));
    }

    Ok(())
}
```

## Features

### Automatic Slot Routing

Muxis automatically routes commands to the correct node based on CRC16 slot calculation (16384 slots).

```rust
// Hash tags ensure keys map to same slot
client.set("{user:1}:name", Bytes::from("Alice")).await?;
client.set("{user:1}:age", Bytes::from("30")).await?;
// Both keys go to same node due to {user:1} hash tag
```

### Topology Discovery

The client automatically discovers cluster topology via `CLUSTER SLOTS` or `CLUSTER NODES`:

```rust
// Manual topology refresh
client.refresh_topology().await?;

// Check cluster status
println!("Nodes: {}", client.node_count().await);
println!("Fully covered: {}", client.is_fully_covered().await);
```

### Connection Pooling

Each cluster node has its own connection pool with health tracking:

- Automatic connection reuse
- Health status monitoring
- Idle connection cleanup
- Per-node connection limits

### Redirect Handling

Muxis handles MOVED and ASK redirects transparently:

**MOVED**: Permanent slot migration (updates topology)
```
Client -> Node A: GET mykey
Node A -> Client: MOVED 12345 127.0.0.1:7001
Client -> Node B: GET mykey (automatically retries)
```

**ASK**: Temporary migration (sends ASKING)
```
Client -> Node A: GET mykey
Node A -> Client: ASK 12345 127.0.0.1:7001
Client -> Node B: ASKING
Client -> Node B: GET mykey (single redirect)
```

### Resilience Features

#### MOVED Storm Detection

Prevents excessive topology refreshes during slot migrations:

```
Threshold: 10 MOVED redirects per second
Cooldown: 500ms minimum between topology refreshes
```

When threshold is exceeded, topology is refreshed once and throttled.

#### IO Error Retry

Automatic retry with exponential backoff on connection failures:

```
Max retries: 3
Backoff: 100ms -> 200ms -> 400ms
```

On IO error:
1. Mark connection as unhealthy
2. Refresh topology to discover new master
3. Retry command with backoff

#### Node Failure Handling

Automatic failover when nodes become unreachable:

1. Detect connection failure via IO errors
2. Mark node as unhealthy in pool
3. Trigger topology refresh
4. Route to new primary node
5. Seamless command execution continues

## Supported Commands

Currently supported cluster commands:

```rust
// String operations
client.get("key").await?;
client.set("key", value).await?;
client.del("key").await?;

// Key operations
client.exists("key").await?;

// Cluster management
client.refresh_topology().await?;
```

## Multi-Key Operations

For operations involving multiple keys, all keys must map to the same slot:

```rust
// Using hash tags to ensure same slot
let keys = vec!["{user:1}:name", "{user:1}:age", "{user:1}:email"];
ClusterClient::validate_same_slot(&keys)?;

// MGET example
client.mget(&keys).await?;
```

**Error**: If keys map to different slots, you'll get a `CROSSSLOT` error.

## Configuration Constants

Cluster behavior is controlled by these constants (defined in code):

```rust
MAX_REDIRECTS = 5               // MOVED/ASK redirect limit
MAX_RETRIES_ON_IO = 3          // Connection failure retry limit
RETRY_DELAY_MS = 100           // Base delay for exponential backoff
MOVED_STORM_THRESHOLD = 10     // Redirects per window to trigger refresh
MOVED_STORM_WINDOW = 1s        // Time window for storm detection
REFRESH_COOLDOWN = 500ms       // Minimum time between refreshes
```

## Cluster Setup

### Using Docker

Quick cluster setup with Docker:

```bash
# Create 6-node cluster (3 masters + 3 replicas)
docker run -d --name redis-cluster -p 7000-7005:7000-7005 \
  redis:7-alpine redis-cli --cluster create \
  127.0.0.1:7000 127.0.0.1:7001 127.0.0.1:7002 \
  127.0.0.1:7003 127.0.0.1:7004 127.0.0.1:7005 \
  --cluster-replicas 1 --cluster-yes
```

### Manual Setup

Create configuration files for each node:

```conf
# redis-7000.conf
port 7000
cluster-enabled yes
cluster-config-file nodes-7000.conf
cluster-node-timeout 5000
appendonly yes
```

Start each node and create cluster:

```bash
redis-server redis-7000.conf &
redis-server redis-7001.conf &
redis-server redis-7002.conf &

redis-cli --cluster create 127.0.0.1:7000 127.0.0.1:7001 127.0.0.1:7002 \
  --cluster-replicas 0
```

## Examples

See `examples/cluster.rs` and `examples/cluster_pipeline.rs` for complete examples.

## Limitations

Current limitations (to be addressed in future releases):

- **Pipelining**: Not yet implemented for cluster mode
- **Pub/Sub**: Not supported in cluster mode
- **Transactions**: MULTI/EXEC not supported across nodes
- **Scripting**: Lua scripts must target single slot
- **Command Coverage**: Only basic commands supported (GET/SET/DEL/EXISTS)

## Troubleshooting

### Connection Issues

```rust
// Check if cluster is reachable
match ClusterClient::connect("127.0.0.1:7000").await {
    Ok(client) => println!("Connected"),
    Err(e) => println!("Connection failed: {}", e),
}
```

### CROSSSLOT Errors

```rust
// Ensure keys use hash tags for multi-key ops
let keys = vec!["{user}:name", "{user}:age"]; // Good
let keys = vec!["user:name", "user:age"];     // Bad - different slots
```

### Topology Issues

```rust
// Manually refresh if topology is stale
client.refresh_topology().await?;

// Verify cluster coverage
if !client.is_fully_covered().await {
    println!("Warning: Not all slots are covered!");
}
```

## Performance Considerations

- **Connection Pooling**: Reuses connections for better performance
- **Topology Caching**: Avoids repeated `CLUSTER SLOTS` calls
- **Smart Routing**: Direct routing to correct node (no redirects in steady state)
- **Retry Throttling**: Prevents retry storms during failures

## Next Steps

- [Architecture](architecture.md) - How cluster routing works internally
- [Testing](testing.md) - Integration testing with clusters
- [Examples](https://github.com/nghiaphamln/muxis/tree/main/examples) - Full code examples
