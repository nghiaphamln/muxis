# Commands Reference

Muxis implements 75+ Redis commands across multiple data structures.

## String Commands

String operations for simple key-value storage.

### GET / SET / DEL

```rust
use bytes::Bytes;

// Basic operations
client.set("mykey", Bytes::from("value")).await?;
let value = client.get("mykey").await?;
client.del("mykey").await?;
```

### SETEX / SETNX

```rust
// Set with expiration (seconds)
client.setex("session", 3600, Bytes::from("data")).await?;

// Set only if not exists
let set = client.setnx("lock:resource", Bytes::from("1")).await?;
```

### MGET / MSET

```rust
// Multiple keys at once
client.mset(&[("key1", "val1"), ("key2", "val2")]).await?;
let values = client.mget(&["key1", "key2"]).await?;
```

### INCR / DECR

```rust
// Atomic increment/decrement
let count = client.incr("counter").await?;
let remaining = client.decr("stock").await?;

// Increment by specific amount
client.incrby("score", 10).await?;
client.incrbyfloat("price", 0.5).await?;
```

### Other String Commands

```rust
// Append to value
client.append("log", Bytes::from("new entry")).await?;

// Get string length
let len = client.strlen("mykey").await?;

// Get and delete atomically
let value = client.getdel("mykey").await?;
```

## Key Commands

Operations that work on keys regardless of their type.

### EXISTS / TYPE

```rust
// Check if key exists
let exists = client.exists("mykey").await?;

// Get key type
let key_type = client.key_type("mykey").await?; // "string", "list", etc.
```

### Expiration

```rust
// Set expiration (seconds)
client.expire("session", 3600).await?;

// Set expiration at Unix timestamp
client.expireat("session", 1640000000).await?;

// Check time-to-live
let ttl = client.ttl("session").await?;

// Remove expiration
client.persist("session").await?;
```

### RENAME / SCAN

```rust
// Rename key
client.rename("oldkey", "newkey").await?;

// Scan keys with cursor
let (cursor, keys) = client.scan(0, None, None).await?;
```

## Hash Commands

Hash operations for field-value pairs within a key.

### HSET / HGET

```rust
use bytes::Bytes;

// Set single field
client.hset("user:1", "name", Bytes::from("Alice")).await?;

// Get single field
let name = client.hget("user:1", "name").await?;
```

### HMSET / HMGET / HGETALL

```rust
// Set multiple fields
client.hmset("user:1", &[("name", "Alice"), ("age", "30")]).await?;

// Get multiple fields
let values = client.hmget("user:1", &["name", "age"]).await?;

// Get all fields and values
let all = client.hgetall("user:1").await?;
```

### Other Hash Commands

```rust
// Delete fields
client.hdel("user:1", &["temp_field"]).await?;

// Check field existence
let exists = client.hexists("user:1", "name").await?;

// Get field count
let count = client.hlen("user:1").await?;

// Get all field names
let fields = client.hkeys("user:1").await?;

// Get all values
let values = client.hvals("user:1").await?;

// Increment field value
client.hincrby("user:1", "visits", 1).await?;
client.hincrbyfloat("user:1", "score", 0.5).await?;
```

## List Commands

List operations for ordered sequences.

### LPUSH / RPUSH / LPOP / RPOP

```rust
use bytes::Bytes;

// Push to left/right
client.lpush("queue", Bytes::from("item1")).await?;
client.rpush("queue", Bytes::from("item2")).await?;

// Pop from left/right
let item = client.lpop("queue").await?;
let item = client.rpop("queue").await?;
```

### LRANGE / LLEN

```rust
// Get range of elements (0-based index, -1 = last)
let items = client.lrange("queue", 0, -1).await?;

// Get list length
let len = client.llen("queue").await?;
```

### Other List Commands

```rust
// Get element by index
let item = client.lindex("queue", 0).await?;

// Set element by index
client.lset("queue", 0, Bytes::from("new_value")).await?;

// Remove elements
client.lrem("queue", 1, Bytes::from("value")).await?;

// Trim list to range
client.ltrim("queue", 0, 99).await?;

// Blocking pop (timeout in seconds)
let item = client.blpop(&["queue1", "queue2"], 5).await?;
```

## Set Commands

Set operations for unordered unique collections.

### SADD / SREM / SMEMBERS

```rust
use bytes::Bytes;

// Add members
client.sadd("tags", &[Bytes::from("rust"), Bytes::from("redis")]).await?;

// Remove members
client.srem("tags", &[Bytes::from("rust")]).await?;

// Get all members
let members = client.smembers("tags").await?;
```

### Set Operations

```rust
// Check membership
let is_member = client.sismember("tags", Bytes::from("rust")).await?;

// Get set size
let count = client.scard("tags").await?;

// Random member
let random = client.srandmember("tags", None).await?;

// Pop random member
let member = client.spop("tags").await?;
```

### Set Combinations

```rust
// Difference
let diff = client.sdiff(&["set1", "set2"]).await?;
client.sdiffstore("result", &["set1", "set2"]).await?;

// Intersection
let inter = client.sinter(&["set1", "set2"]).await?;
client.sinterstore("result", &["set1", "set2"]).await?;

// Union
let union = client.sunion(&["set1", "set2"]).await?;
client.sunionstore("result", &["set1", "set2"]).await?;
```

## Sorted Set Commands

Sorted set operations with scores.

### ZADD / ZREM

```rust
use bytes::Bytes;

// Add members with scores
client.zadd("leaderboard", &[(100.0, Bytes::from("player1")), (200.0, Bytes::from("player2"))]).await?;

// Remove members
client.zrem("leaderboard", &[Bytes::from("player1")]).await?;
```

### ZRANGE / ZRANK

```rust
// Get range by rank (0-based)
let members = client.zrange("leaderboard", 0, 9).await?; // Top 10

// Get member rank
let rank = client.zrank("leaderboard", Bytes::from("player1")).await?;

// Get member score
let score = client.zscore("leaderboard", Bytes::from("player1")).await?;
```

### Range Operations

```rust
// Range by score
let members = client.zrangebyscore("leaderboard", 100.0, 200.0).await?;

// Count in score range
let count = client.zcount("leaderboard", 100.0, 200.0).await?;

// Remove by rank
client.zremrangebyrank("leaderboard", 0, 9).await?;

// Remove by score
client.zremrangebyscore("leaderboard", 0.0, 100.0).await?;
```

### Other Sorted Set Commands

```rust
// Get set size
let count = client.zcard("leaderboard").await?;

// Increment score
client.zincrby("leaderboard", 10.0, Bytes::from("player1")).await?;

// Reverse range
let bottom = client.zrevrange("leaderboard", 0, 9).await?;

// Pop min/max
let min = client.zpopmin("leaderboard", 1).await?;
let max = client.zpopmax("leaderboard", 1).await?;

// Blocking pop
let item = client.bzpopmin(&["leaderboard"], 5).await?;
```

## Connection Commands

Server and connection management.

```rust
// Ping server
let pong = client.ping().await?;

// Echo message
let echo = client.echo(Bytes::from("hello")).await?;

// Authenticate
client.auth("password").await?;

// Select database
client.select(2).await?;

// Set client name
client.client_setname("my-app").await?;
```

## Cluster Commands

See [Cluster Mode](cluster.md) for cluster-specific commands and operations.
