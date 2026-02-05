# Getting Started

This guide will help you get started with Muxis Redis client.

## Installation

Add Muxis to your `Cargo.toml`:

```toml
[dependencies]
muxis = "0.4"

# Optional features
muxis = { version = "0.4", features = ["cluster", "tls"] }
```

### Feature Flags

- `cluster` - Redis Cluster support with automatic slot routing
- `tls` - TLS/SSL support for secure connections
- `resp3` - RESP3 protocol support (experimental)
- `json` - JSON serialization helpers
- `streams` - Redis Streams support
- `test-utils` - Testing utilities

## Quick Start

### Basic Connection

```rust
use bytes::Bytes;
use muxis::{Client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to Redis
    let mut client = Client::connect("redis://127.0.0.1:6379").await?;

    // Set a value
    client.set("mykey", Bytes::from("Hello, Muxis!")).await?;

    // Get a value
    if let Some(value) = client.get("mykey").await? {
        println!("Value: {}", String::from_utf8_lossy(&value));
    }

    Ok(())
}
```

### Using ClientBuilder

For more configuration options:

```rust
use muxis::ClientBuilder;

let mut client = ClientBuilder::new()
    .address("redis://127.0.0.1:6379")
    .database(2)
    .password("secret")
    .build()
    .await?;
```

### Connection URLs

Muxis supports standard Redis URL format:

```
redis://[:password@]host[:port][/database]
rediss://[:password@]host[:port][/database]  # TLS
```

Examples:
- `redis://127.0.0.1` - Default port 6379, database 0
- `redis://127.0.0.1:6380` - Custom port
- `redis://:mypassword@127.0.0.1` - With password
- `redis://127.0.0.1/2` - Select database 2
- `rediss://127.0.0.1` - TLS connection

## Basic Operations

### String Operations

```rust
use bytes::Bytes;

// SET and GET
client.set("user:1:name", Bytes::from("Alice")).await?;
let name = client.get("user:1:name").await?;

// SET with expiration (seconds)
client.setex("session:123", 3600, Bytes::from("data")).await?;

// Multiple keys
client.mset(&[("key1", "val1"), ("key2", "val2")]).await?;
let values = client.mget(&["key1", "key2"]).await?;
```

### Atomic Operations

```rust
// Increment/decrement
let views = client.incr("page:views").await?;
let likes = client.decr("post:dislikes").await?;

// Increment by amount
client.incrby("score", 10).await?;
```

### Key Management

```rust
// Check existence
let exists = client.exists("mykey").await?;

// Set expiration (seconds)
client.expire("mykey", 3600).await?;

// Check TTL
let ttl = client.ttl("mykey").await?;

// Delete keys
client.del("key1").await?;
```

## Error Handling

Muxis uses `Result<T, muxis::Error>` for all operations:

```rust
use muxis::Error;

match client.get("mykey").await {
    Ok(Some(value)) => println!("Value: {:?}", value),
    Ok(None) => println!("Key not found"),
    Err(Error::Io { source }) => println!("Connection error: {}", source),
    Err(Error::Server { message }) => println!("Redis error: {}", message),
    Err(e) => println!("Other error: {}", e),
}
```

## Next Steps

- [Commands Reference](commands.md) - All available commands
- [Cluster Mode](cluster.md) - Redis Cluster support
- [Multiplexing](multiplexing.md) - Concurrent requests
- [Architecture](architecture.md) - How Muxis works
