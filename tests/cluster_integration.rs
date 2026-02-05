//! Integration tests for Redis Cluster operations.
//!
//! These tests require a real Redis Cluster running on localhost.
//! All tests are marked with #[ignore] by default.
//!
//! Setup with Docker:
//! ```bash
//! docker run -d --name redis-cluster \
//!   -p 7000-7005:7000-7005 \
//!   grokzen/redis-cluster:latest
//! ```
//!
//! Run tests:
//! ```bash
//! cargo test --test cluster_integration --features cluster -- --ignored
//! ```

#![cfg(feature = "cluster")]

use bytes::Bytes;
use muxis::cluster::ClusterClient;
use muxis::Result;

/// Helper function to create a cluster client for testing.
async fn create_test_client() -> Result<ClusterClient> {
    ClusterClient::connect("127.0.0.1:7000,127.0.0.1:7001,127.0.0.1:7002").await
}

#[tokio::test]
#[ignore]
async fn test_cluster_connect() {
    let client = create_test_client().await.expect("failed to connect");
    
    // Verify topology was discovered
    let node_count = client.node_count().await;
    assert!(node_count >= 3, "expected at least 3 nodes, got {}", node_count);
    
    // Verify all slots are covered
    let is_covered = client.is_fully_covered().await;
    assert!(is_covered, "cluster should cover all 16384 slots");
}

#[tokio::test]
#[ignore]
async fn test_cluster_basic_operations() {
    let client = create_test_client().await.expect("failed to connect");
    
    let key = "integration:test:basic";
    let value = Bytes::from("Hello, Cluster!");
    
    // SET operation
    client.set(key, value.clone()).await.expect("SET failed");
    
    // GET operation
    let retrieved = client.get(key).await.expect("GET failed");
    assert_eq!(retrieved, Some(value), "retrieved value should match");
    
    // EXISTS operation
    let exists = client.exists(key).await.expect("EXISTS failed");
    assert!(exists, "key should exist");
    
    // DEL operation
    let deleted = client.del(key).await.expect("DEL failed");
    assert_eq!(deleted, 1, "should delete 1 key");
    
    // Verify deletion
    let exists_after = client.exists(key).await.expect("EXISTS failed");
    assert!(!exists_after, "key should not exist after deletion");
}

#[tokio::test]
#[ignore]
async fn test_cluster_hash_tags() {
    let client = create_test_client().await.expect("failed to connect");
    
    // Keys with same hash tag should map to same slot
    let keys = vec![
        "user:{12345}:name",
        "user:{12345}:email",
        "user:{12345}:age",
    ];
    
    // Validate they map to same slot
    let slot = ClusterClient::validate_same_slot(&keys)
        .expect("keys with same hash tag should map to same slot");
    
    println!("All keys map to slot: {}", slot);
    
    // Set all values
    for (i, key) in keys.iter().enumerate() {
        let value = format!("value_{}", i);
        client.set(key, Bytes::from(value))
            .await
            .expect("SET with hash tag failed");
    }
    
    // Retrieve all values
    for (i, key) in keys.iter().enumerate() {
        let expected = format!("value_{}", i);
        let retrieved = client.get(key).await.expect("GET failed");
        assert_eq!(
            retrieved,
            Some(Bytes::from(expected)),
            "value should match for key: {}",
            key
        );
    }
    
    // Clean up
    for key in &keys {
        client.del(key).await.expect("DEL failed");
    }
}

#[tokio::test]
#[ignore]
async fn test_cluster_multiple_keys_different_slots() {
    let client = create_test_client().await.expect("failed to connect");
    
    // These keys likely map to different slots
    let keys = vec![
        "test:key:1",
        "test:key:2",
        "test:key:3",
        "test:key:4",
        "test:key:5",
    ];
    
    // Set all keys
    for (i, key) in keys.iter().enumerate() {
        let value = format!("value_{}", i);
        client.set(key, Bytes::from(value))
            .await
            .expect("SET failed");
    }
    
    // Get all keys
    for (i, key) in keys.iter().enumerate() {
        let expected = format!("value_{}", i);
        let retrieved = client.get(key).await.expect("GET failed");
        assert_eq!(
            retrieved,
            Some(Bytes::from(expected)),
            "value should match for key: {}",
            key
        );
    }
    
    // Delete all keys
    for key in &keys {
        let deleted = client.del(key).await.expect("DEL failed");
        assert_eq!(deleted, 1, "should delete 1 key");
    }
}

#[tokio::test]
#[ignore]
async fn test_cluster_topology_refresh() {
    let client = create_test_client().await.expect("failed to connect");
    
    let node_count_before = client.node_count().await;
    let is_covered_before = client.is_fully_covered().await;
    
    // Refresh topology
    client.refresh_topology().await.expect("refresh failed");
    
    let node_count_after = client.node_count().await;
    let is_covered_after = client.is_fully_covered().await;
    
    // Topology should remain consistent after refresh
    assert_eq!(
        node_count_before, node_count_after,
        "node count should not change after refresh"
    );
    assert_eq!(
        is_covered_before, is_covered_after,
        "coverage should not change after refresh"
    );
}

#[tokio::test]
#[ignore]
async fn test_cluster_concurrent_operations() {
    let client = create_test_client().await.expect("failed to connect");
    
    let mut handles = vec![];
    
    // Spawn 10 concurrent tasks
    for i in 0..10 {
        let client_clone = client.clone();
        let handle = tokio::spawn(async move {
            let key = format!("concurrent:test:{}", i);
            let value = Bytes::from(format!("value_{}", i));
            
            // SET
            client_clone.set(&key, value.clone()).await?;
            
            // GET
            let retrieved = client_clone.get(&key).await?;
            assert_eq!(retrieved, Some(value));
            
            // DEL
            let deleted = client_clone.del(&key).await?;
            assert_eq!(deleted, 1);
            
            Ok::<_, muxis::Error>(())
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.expect("task panicked").expect("operation failed");
    }
}

#[tokio::test]
#[ignore]
async fn test_cluster_stress_operations() {
    let client = create_test_client().await.expect("failed to connect");
    
    let operation_count = 100;
    
    // Perform many operations to stress test routing and connections
    for i in 0..operation_count {
        let key = format!("stress:test:{}", i);
        let value = Bytes::from(format!("value_{}", i));
        
        // SET
        client.set(&key, value.clone()).await.expect("SET failed");
        
        // GET
        let retrieved = client.get(&key).await.expect("GET failed");
        assert_eq!(retrieved, Some(value), "value mismatch at iteration {}", i);
        
        // EXISTS
        let exists = client.exists(&key).await.expect("EXISTS failed");
        assert!(exists, "key should exist at iteration {}", i);
        
        // DEL
        let deleted = client.del(&key).await.expect("DEL failed");
        assert_eq!(deleted, 1, "should delete 1 key at iteration {}", i);
    }
}

#[tokio::test]
#[ignore]
async fn test_cluster_nonexistent_key() {
    let client = create_test_client().await.expect("failed to connect");
    
    let key = "nonexistent:key:test";
    
    // GET nonexistent key
    let result = client.get(key).await.expect("GET failed");
    assert_eq!(result, None, "nonexistent key should return None");
    
    // EXISTS nonexistent key
    let exists = client.exists(key).await.expect("EXISTS failed");
    assert!(!exists, "nonexistent key should not exist");
    
    // DEL nonexistent key
    let deleted = client.del(key).await.expect("DEL failed");
    assert_eq!(deleted, 0, "deleting nonexistent key should return 0");
}

#[tokio::test]
#[ignore]
async fn test_cluster_large_value() {
    let client = create_test_client().await.expect("failed to connect");
    
    let key = "large:value:test";
    
    // Create a large value (1 MB)
    let large_value = vec![b'x'; 1024 * 1024];
    let value = Bytes::from(large_value);
    
    // SET large value
    client.set(key, value.clone()).await.expect("SET failed");
    
    // GET large value
    let retrieved = client.get(key).await.expect("GET failed");
    assert_eq!(retrieved, Some(value), "large value should match");
    
    // Clean up
    client.del(key).await.expect("DEL failed");
}

#[tokio::test]
#[ignore]
async fn test_cluster_empty_value() {
    let client = create_test_client().await.expect("failed to connect");
    
    let key = "empty:value:test";
    let value = Bytes::from("");
    
    // SET empty value
    client.set(key, value.clone()).await.expect("SET failed");
    
    // GET empty value
    let retrieved = client.get(key).await.expect("GET failed");
    assert_eq!(retrieved, Some(value), "empty value should be retrievable");
    
    // Clean up
    client.del(key).await.expect("DEL failed");
}

#[tokio::test]
#[ignore]
async fn test_cluster_special_characters() {
    let client = create_test_client().await.expect("failed to connect");
    
    let keys = vec![
        "key:with:colons",
        "key/with/slashes",
        "key-with-dashes",
        "key_with_underscores",
        "key.with.dots",
    ];
    
    for key in &keys {
        let value = Bytes::from(format!("value_for_{}", key));
        
        // SET
        client.set(key, value.clone()).await.expect("SET failed");
        
        // GET
        let retrieved = client.get(key).await.expect("GET failed");
        assert_eq!(retrieved, Some(value), "value mismatch for key: {}", key);
        
        // DEL
        client.del(key).await.expect("DEL failed");
    }
}
