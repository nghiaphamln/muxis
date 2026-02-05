//! Example: Pipelined operations in Redis Cluster.
//!
//! This example demonstrates batching multiple operations to the same slot
//! for improved performance in Redis Cluster.
//!
//! Requirements:
//! - Redis Cluster running on localhost (ports 7000-7005)
//! - Enable cluster feature: cargo run --example cluster_pipeline --features cluster
//!
//! Quick setup with Docker:
//! ```bash
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
    println!("=== Muxis Cluster Pipeline Example ===\n");

    // Connect to cluster
    println!("1. Connecting to Redis Cluster...");
    let client = ClusterClient::connect("127.0.0.1:7000,127.0.0.1:7001,127.0.0.1:7002").await?;
    println!("   Connected successfully!");

    // Example 1: Batch operations on same-slot keys using hash tags
    println!("\n2. Batch operations on same-slot keys...");

    // All these keys map to the same slot because of {user:1000}
    let user_keys = vec![
        ("profile:{user:1000}:name", "Alice"),
        ("profile:{user:1000}:email", "alice@example.com"),
        ("profile:{user:1000}:age", "30"),
        ("profile:{user:1000}:city", "San Francisco"),
        ("profile:{user:1000}:country", "USA"),
    ];

    // Validate all keys map to same slot
    let keys_only: Vec<&str> = user_keys.iter().map(|(k, _)| *k).collect();
    let slot = ClusterClient::validate_same_slot(&keys_only)?;
    println!("   All user profile keys map to slot: {}", slot);

    // Set all values (these go to the same node, can be pipelined)
    println!("   Setting {} keys in batch...", user_keys.len());
    let start = std::time::Instant::now();

    // Use futures to execute all SETs concurrently
    let mut set_futures = vec![];
    for (key, value) in &user_keys {
        set_futures.push(client.set(key, Bytes::from(*value)));
    }

    // Wait for all SETs to complete
    for future in set_futures {
        future.await?;
    }

    let duration = start.elapsed();
    println!("   Batch SET completed in {:?}", duration);

    // Get all values (pipelined reads)
    println!("   Reading {} keys in batch...", user_keys.len());
    let start = std::time::Instant::now();

    let mut get_futures = vec![];
    for (key, _) in &user_keys {
        get_futures.push(client.get(key));
    }

    // Wait for all GETs
    let mut results = vec![];
    for future in get_futures {
        results.push(future.await?);
    }

    let duration = start.elapsed();
    println!("   Batch GET completed in {:?}", duration);

    // Verify results
    for (i, (key, expected_value)) in user_keys.iter().enumerate() {
        if let Some(value) = &results[i] {
            let value_str = String::from_utf8_lossy(value);
            assert_eq!(value_str, *expected_value);
            println!("   ✓ {}: {}", key, value_str);
        }
    }

    // Example 2: Compare with sequential operations
    println!("\n3. Comparing batch vs sequential operations...");

    let test_keys = vec![
        ("test:{batch:1}:a", "value_a"),
        ("test:{batch:1}:b", "value_b"),
        ("test:{batch:1}:c", "value_c"),
        ("test:{batch:1}:d", "value_d"),
        ("test:{batch:1}:e", "value_e"),
    ];

    // Sequential operations
    println!("   Sequential SET operations...");
    let start = std::time::Instant::now();
    for (key, value) in &test_keys {
        client.set(key, Bytes::from(*value)).await?;
    }
    let sequential_duration = start.elapsed();
    println!("   Sequential: {:?}", sequential_duration);

    // Batch operations
    println!("   Concurrent SET operations...");
    let start = std::time::Instant::now();
    let mut futures = vec![];
    for (key, value) in &test_keys {
        futures.push(client.set(key, Bytes::from(*value)));
    }
    for future in futures {
        future.await?;
    }
    let batch_duration = start.elapsed();
    println!("   Concurrent: {:?}", batch_duration);

    let speedup = sequential_duration.as_micros() as f64 / batch_duration.as_micros() as f64;
    println!("   Speedup: {:.2}x faster", speedup);

    // Example 3: Mixed operations on same slot
    println!("\n4. Mixed operations (SET, GET, EXISTS, DEL) on same slot...");

    let mixed_keys = vec![
        "session:{abc123}:token",
        "session:{abc123}:user_id",
        "session:{abc123}:expires",
        "session:{abc123}:device",
    ];

    // Validate same slot
    let slot = ClusterClient::validate_same_slot(&mixed_keys)?;
    println!("   All session keys map to slot: {}", slot);

    // Concurrent mixed operations
    let start = std::time::Instant::now();

    let mut futures = vec![];

    // SET operations
    futures.push(tokio::spawn({
        let client = client.clone();
        async move {
            client
                .set("session:{abc123}:token", Bytes::from("xyz789"))
                .await
        }
    }));

    futures.push(tokio::spawn({
        let client = client.clone();
        async move {
            client
                .set("session:{abc123}:user_id", Bytes::from("12345"))
                .await
        }
    }));

    futures.push(tokio::spawn({
        let client = client.clone();
        async move {
            client
                .set("session:{abc123}:expires", Bytes::from("3600"))
                .await
        }
    }));

    futures.push(tokio::spawn({
        let client = client.clone();
        async move {
            client
                .set("session:{abc123}:device", Bytes::from("mobile"))
                .await
        }
    }));

    // Wait for all operations
    for future in futures {
        future.await.unwrap()?;
    }

    let duration = start.elapsed();
    println!("   All operations completed in {:?}", duration);

    // Verify with EXISTS
    for key in &mixed_keys {
        let exists = client.exists(key).await?;
        assert!(exists, "key should exist: {}", key);
    }
    println!("   ✓ All keys verified to exist");

    // Clean up all keys
    println!("\n5. Cleaning up test data...");

    let all_keys: Vec<_> = user_keys
        .iter()
        .map(|(k, _)| *k)
        .chain(test_keys.iter().map(|(k, _)| *k))
        .chain(mixed_keys.iter().copied())
        .collect();

    let mut del_futures = vec![];
    for key in &all_keys {
        del_futures.push(client.del(key));
    }

    let mut total_deleted = 0;
    for future in del_futures {
        total_deleted += future.await?;
    }

    println!("   Deleted {} keys", total_deleted);

    println!("\n=== Example Complete ===");
    println!("\nKey takeaways:");
    println!("  ✓ Use hash tags to group related keys on same slot");
    println!("  ✓ Batch operations on same-slot keys for better performance");
    println!("  ✓ Concurrent operations can provide significant speedup");
    println!("  ✓ ClusterClient handles routing automatically");
    println!("  ✓ validate_same_slot() helps ensure CROSSSLOT safety");

    Ok(())
}

#[cfg(not(feature = "cluster"))]
fn main() {
    eprintln!("Error: This example requires the 'cluster' feature.");
    eprintln!("Run with: cargo run --example cluster_pipeline --features cluster");
    std::process::exit(1);
}
