//! Example: Redis Cluster operations with automatic redirect handling.
//!
//! This example demonstrates how to use the ClusterClient for Redis Cluster operations.
//!
//! Requirements:
//! - Redis Cluster running on localhost (ports 7000-7005)
//! - Enable cluster feature: cargo run --example cluster --features cluster
//!
//! Quick setup with Docker:
//! ```bash
//! # Create a 6-node Redis Cluster
//! docker run -d --name redis-cluster -p 7000-7005:7000-7005 \
//!   grokzen/redis-cluster:latest
//! ```

#[cfg(feature = "cluster")]
use bytes::Bytes;
#[cfg(feature = "cluster")]
use muxis::cluster::ClusterClient;

#[cfg(feature = "cluster")]
#[tokio::main]
async fn main() -> muxis::Result<()> {
    println!("=== Muxis Redis Cluster Example ===\n");

    // Connect to Redis Cluster using seed nodes
    println!("1. Connecting to Redis Cluster...");
    let client = ClusterClient::connect("127.0.0.1:7000,127.0.0.1:7001,127.0.0.1:7002").await?;
    println!("   Connected successfully!");

    // Check cluster topology
    println!("\n2. Checking cluster topology...");
    let node_count = client.node_count().await;
    let is_covered = client.is_fully_covered().await;
    println!("   Cluster nodes: {}", node_count);
    println!("   All slots covered: {}", is_covered);

    if !is_covered {
        println!("   Warning: Cluster does not cover all 16384 slots!");
    }

    // Basic operations with automatic slot routing
    println!("\n3. Testing basic operations (automatic routing)...");

    // SET operation
    println!("   Setting key 'user:1000:name' = 'Alice'");
    client
        .set("user:1000:name", Bytes::from("Alice"))
        .await?;

    // GET operation
    println!("   Getting key 'user:1000:name'");
    if let Some(value) = client.get("user:1000:name").await? {
        println!("   Value: {}", String::from_utf8_lossy(&value));
    }

    // EXISTS operation
    println!("   Checking if key 'user:1000:name' exists");
    let exists = client.exists("user:1000:name").await?;
    println!("   Exists: {}", exists);

    // DEL operation
    println!("   Deleting key 'user:1000:name'");
    let deleted = client.del("user:1000:name").await?;
    println!("   Deleted: {} keys", deleted);

    // Verify deletion
    let exists_after = client.exists("user:1000:name").await?;
    println!("   Exists after delete: {}", exists_after);

    // Demonstrate hash tag for multi-key operations
    println!("\n4. Testing hash tags for same-slot keys...");

    // Keys with same hash tag {user:1000} will map to same slot
    let keys_with_tag = vec![
        "profile:{user:1000}:name",
        "profile:{user:1000}:email",
        "profile:{user:1000}:age",
    ];

    // Validate they map to same slot
    match ClusterClient::validate_same_slot(&keys_with_tag) {
        Ok(slot) => println!("   All keys map to slot: {}", slot),
        Err(e) => println!("   Validation failed: {:?}", e),
    }

    // Set values using hash tags
    for (i, key) in keys_with_tag.iter().enumerate() {
        let value = format!("value_{}", i);
        client.set(key, Bytes::from(value)).await?;
        println!("   Set key: {}", key);
    }

    // Retrieve values
    for key in &keys_with_tag {
        if let Some(value) = client.get(key).await? {
            println!("   Get {}: {}", key, String::from_utf8_lossy(&value));
        }
    }

    // Clean up
    for key in &keys_with_tag {
        client.del(key).await?;
    }

    // Demonstrate redirect handling (during slot migration)
    println!("\n5. Redirect handling...");
    println!("   MOVED redirects: Handled automatically on slot migration");
    println!("   ASK redirects: Handled automatically during migration");
    println!("   Max redirects: 5 (prevents infinite loops)");

    // Test a few operations to demonstrate redirect handling works
    for i in 0..5 {
        let key = format!("test:key:{}", i);
        client.set(&key, Bytes::from(format!("value_{}", i))).await?;
        client.get(&key).await?;
        client.del(&key).await?;
    }
    println!("   Completed 5 operations with redirect handling");

    // Demonstrate CROSSSLOT error prevention
    println!("\n6. Testing CROSSSLOT validation...");
    let different_keys = vec!["key1", "key2", "key3"];

    match ClusterClient::validate_same_slot(&different_keys) {
        Ok(slot) => println!("   All keys map to slot: {} (lucky!)", slot),
        Err(muxis::Error::CrossSlot) => {
            println!("   CROSSSLOT error prevented: keys map to different slots");
        }
        Err(e) => println!("   Unexpected error: {:?}", e),
    }

    // Show how to use hash tags to avoid CROSSSLOT
    println!("\n7. Using hash tags to avoid CROSSSLOT...");
    let same_slot_keys = vec!["{myapp}:key1", "{myapp}:key2", "{myapp}:key3"];

    match ClusterClient::validate_same_slot(&same_slot_keys) {
        Ok(slot) => {
            println!("   Success! All keys with {{myapp}} hash tag map to slot: {}", slot);

            // Now these operations would work in a multi-key context
            for key in &same_slot_keys {
                client.set(key, Bytes::from("value")).await?;
            }
            println!("   Set all keys successfully");

            // Clean up
            for key in &same_slot_keys {
                client.del(key).await?;
            }
        }
        Err(e) => println!("   Unexpected error: {:?}", e),
    }

    // Topology refresh
    println!("\n8. Testing topology refresh...");
    client.refresh_topology().await?;
    println!("   Topology refreshed successfully");

    let node_count_after = client.node_count().await;
    println!("   Cluster nodes after refresh: {}", node_count_after);

    println!("\n=== Example Complete ===");
    println!("\nKey features demonstrated:");
    println!("  ✓ Automatic slot-based routing");
    println!("  ✓ MOVED/ASK redirect handling");
    println!("  ✓ Hash tags for same-slot keys");
    println!("  ✓ CROSSSLOT error prevention");
    println!("  ✓ Topology discovery and refresh");
    println!("  ✓ Connection pooling per node");

    Ok(())
}

#[cfg(not(feature = "cluster"))]
fn main() {
    eprintln!("Error: This example requires the 'cluster' feature.");
    eprintln!("Run with: cargo run --example cluster --features cluster");
    std::process::exit(1);
}
