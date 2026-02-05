//! Stress tests for cluster redirect handling.
//!
//! These tests simulate high-load scenarios with redirects to verify
//! the robustness of MOVED/ASK handling logic.
//!
//! Requirements:
//! - Redis Cluster running on localhost (ports 7000-7005)
//!
//! Run with:
//! ```bash
//! cargo test --test cluster_stress --features cluster -- --ignored
//! ```

#![cfg(feature = "cluster")]

use bytes::Bytes;
use muxis::cluster::ClusterClient;
use muxis::Result;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Helper to create test client.
async fn create_test_client() -> Result<ClusterClient> {
    ClusterClient::connect("127.0.0.1:7000,127.0.0.1:7001,127.0.0.1:7002").await
}

#[tokio::test]
#[ignore]
async fn stress_test_many_keys_all_slots() {
    let client = create_test_client().await.expect("failed to connect");
    
    // Test with 1000 keys that will map to different slots
    let num_keys = 1000;
    println!("Testing {} keys across all slots...", num_keys);
    
    let start = std::time::Instant::now();
    
    // SET phase
    for i in 0..num_keys {
        let key = format!("stress:allslots:{}", i);
        let value = Bytes::from(format!("value_{}", i));
        client.set(&key, value).await.expect("SET failed");
    }
    
    let set_duration = start.elapsed();
    println!("SET {} keys: {:?}", num_keys, set_duration);
    
    // GET phase
    let start = std::time::Instant::now();
    for i in 0..num_keys {
        let key = format!("stress:allslots:{}", i);
        let value = client.get(&key).await.expect("GET failed");
        assert!(value.is_some(), "value should exist for key {}", i);
    }
    
    let get_duration = start.elapsed();
    println!("GET {} keys: {:?}", num_keys, get_duration);
    
    // DEL phase
    let start = std::time::Instant::now();
    for i in 0..num_keys {
        let key = format!("stress:allslots:{}", i);
        client.del(&key).await.expect("DEL failed");
    }
    
    let del_duration = start.elapsed();
    println!("DEL {} keys: {:?}", num_keys, del_duration);
    
    println!("Total operations: {}", num_keys * 3);
    println!("Total time: {:?}", set_duration + get_duration + del_duration);
}

#[tokio::test]
#[ignore]
async fn stress_test_concurrent_operations() {
    let client = create_test_client().await.expect("failed to connect");
    
    let num_tasks = 50;
    let ops_per_task = 20;
    
    println!("Running {} concurrent tasks with {} ops each...", num_tasks, ops_per_task);
    
    let start = std::time::Instant::now();
    let mut handles = vec![];
    
    for task_id in 0..num_tasks {
        let client_clone = client.clone();
        let handle = tokio::spawn(async move {
            for op_id in 0..ops_per_task {
                let key = format!("stress:concurrent:{}:{}", task_id, op_id);
                let value = Bytes::from(format!("value_{}_{}", task_id, op_id));
                
                // SET
                client_clone.set(&key, value.clone()).await?;
                
                // GET
                let retrieved = client_clone.get(&key).await?;
                assert_eq!(retrieved, Some(value));
                
                // DEL
                let deleted = client_clone.del(&key).await?;
                assert_eq!(deleted, 1);
            }
            
            Ok::<_, muxis::Error>(())
        });
        handles.push(handle);
    }
    
    // Wait for all tasks
    for handle in handles {
        handle.await.expect("task panicked").expect("operation failed");
    }
    
    let duration = start.elapsed();
    let total_ops = num_tasks * ops_per_task * 3; // SET + GET + DEL
    println!("Completed {} operations in {:?}", total_ops, duration);
    println!("Ops/sec: {:.2}", total_ops as f64 / duration.as_secs_f64());
}

#[tokio::test]
#[ignore]
async fn stress_test_rapid_topology_refresh() {
    let client = create_test_client().await.expect("failed to connect");
    
    let num_refreshes = 100;
    
    println!("Testing {} rapid topology refreshes...", num_refreshes);
    
    let start = std::time::Instant::now();
    
    for i in 0..num_refreshes {
        client.refresh_topology().await.expect("refresh failed");
        
        // Interleave with actual operations
        let key = format!("stress:refresh:{}", i);
        client.set(&key, Bytes::from("value")).await.expect("SET failed");
        client.get(&key).await.expect("GET failed");
        client.del(&key).await.expect("DEL failed");
    }
    
    let duration = start.elapsed();
    println!("Completed in {:?}", duration);
    println!("Avg per refresh+ops: {:?}", duration / num_refreshes);
}

#[tokio::test]
#[ignore]
async fn stress_test_same_key_contention() {
    let client = create_test_client().await.expect("failed to connect");
    
    let num_tasks = 20;
    let ops_per_task = 50;
    let shared_key = "stress:contention:shared";
    
    println!("Testing {} tasks contending on same key...", num_tasks);
    
    let counter = Arc::new(AtomicU64::new(0));
    let start = std::time::Instant::now();
    
    let mut handles = vec![];
    
    for task_id in 0..num_tasks {
        let client_clone = client.clone();
        let counter_clone = counter.clone();
        
        let handle = tokio::spawn(async move {
            for _ in 0..ops_per_task {
                let count = counter_clone.fetch_add(1, Ordering::Relaxed);
                let value = Bytes::from(format!("value_{}_{}", task_id, count));
                
                // All tasks write to same key (last write wins)
                client_clone.set(shared_key, value).await?;
                
                // Read back
                let _retrieved = client_clone.get(shared_key).await?;
            }
            
            Ok::<_, muxis::Error>(())
        });
        handles.push(handle);
    }
    
    // Wait for all tasks
    for handle in handles {
        handle.await.expect("task panicked").expect("operation failed");
    }
    
    let duration = start.elapsed();
    let total_ops = num_tasks * ops_per_task * 2; // SET + GET
    println!("Completed {} operations in {:?}", total_ops, duration);
    
    // Cleanup
    client.del(shared_key).await.expect("cleanup failed");
}

#[tokio::test]
#[ignore]
async fn stress_test_hash_tag_batching() {
    let client = create_test_client().await.expect("failed to connect");
    
    let num_batches = 100;
    let keys_per_batch = 10;
    
    println!("Testing {} batches with {} keys each...", num_batches, keys_per_batch);
    
    let start = std::time::Instant::now();
    
    for batch_id in 0..num_batches {
        // All keys in this batch map to same slot
        let keys: Vec<String> = (0..keys_per_batch)
            .map(|i| format!("stress:batch:{{{}}}:{}", batch_id, i))
            .collect();
        
        // Verify same slot
        let key_refs: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();
        ClusterClient::validate_same_slot(&key_refs).expect("should be same slot");
        
        // SET all keys concurrently
        let mut set_futures = vec![];
        for key in &keys {
            set_futures.push(client.set(key, Bytes::from("value")));
        }
        for future in set_futures {
            future.await.expect("SET failed");
        }
        
        // GET all keys concurrently
        let mut get_futures = vec![];
        for key in &keys {
            get_futures.push(client.get(key));
        }
        for future in get_futures {
            future.await.expect("GET failed");
        }
        
        // DEL all keys concurrently
        let mut del_futures = vec![];
        for key in &keys {
            del_futures.push(client.del(key));
        }
        for future in del_futures {
            future.await.expect("DEL failed");
        }
    }
    
    let duration = start.elapsed();
    let total_ops = num_batches * keys_per_batch * 3; // SET + GET + DEL
    println!("Completed {} operations in {:?}", total_ops, duration);
    println!("Ops/sec: {:.2}", total_ops as f64 / duration.as_secs_f64());
}

#[tokio::test]
#[ignore]
async fn stress_test_sustained_load() {
    let client = create_test_client().await.expect("failed to connect");
    
    let duration_secs = 10;
    let num_workers = 10;
    
    println!("Running sustained load test for {} seconds with {} workers...", 
             duration_secs, num_workers);
    
    let should_stop = Arc::new(AtomicU64::new(0));
    let total_ops = Arc::new(AtomicU64::new(0));
    
    let start = std::time::Instant::now();
    let mut handles = vec![];
    
    for worker_id in 0..num_workers {
        let client_clone = client.clone();
        let should_stop_clone = should_stop.clone();
        let total_ops_clone = total_ops.clone();
        
        let handle = tokio::spawn(async move {
            let mut local_ops = 0u64;
            
            while should_stop_clone.load(Ordering::Relaxed) == 0 {
                let key = format!("stress:sustained:{}:{}", worker_id, local_ops);
                let value = Bytes::from(format!("value_{}", local_ops));
                
                if let Ok(()) = client_clone.set(&key, value).await {
                    if let Ok(_) = client_clone.get(&key).await {
                        if let Ok(_) = client_clone.del(&key).await {
                            local_ops += 3; // SET + GET + DEL
                        }
                    }
                }
            }
            
            total_ops_clone.fetch_add(local_ops, Ordering::Relaxed);
            Ok::<_, muxis::Error>(())
        });
        handles.push(handle);
    }
    
    // Run for specified duration
    tokio::time::sleep(Duration::from_secs(duration_secs)).await;
    
    // Signal stop
    should_stop.store(1, Ordering::Relaxed);
    
    // Wait for all workers
    for handle in handles {
        handle.await.expect("task panicked").expect("operation failed");
    }
    
    let actual_duration = start.elapsed();
    let ops = total_ops.load(Ordering::Relaxed);
    
    println!("Sustained load results:");
    println!("  Duration: {:?}", actual_duration);
    println!("  Total operations: {}", ops);
    println!("  Ops/sec: {:.2}", ops as f64 / actual_duration.as_secs_f64());
    println!("  Ops/sec/worker: {:.2}", ops as f64 / actual_duration.as_secs_f64() / num_workers as f64);
}

#[tokio::test]
#[ignore]
async fn stress_test_large_values() {
    let client = create_test_client().await.expect("failed to connect");
    
    let sizes = vec![
        1024,           // 1 KB
        10 * 1024,      // 10 KB
        100 * 1024,     // 100 KB
        1024 * 1024,    // 1 MB
    ];
    
    for size in sizes {
        println!("Testing with {} byte values...", size);
        
        let num_keys = 10;
        let value = Bytes::from(vec![b'x'; size]);
        
        let start = std::time::Instant::now();
        
        // SET
        for i in 0..num_keys {
            let key = format!("stress:large:{}:{}", size, i);
            client.set(&key, value.clone()).await.expect("SET failed");
        }
        
        // GET
        for i in 0..num_keys {
            let key = format!("stress:large:{}:{}", size, i);
            let retrieved = client.get(&key).await.expect("GET failed");
            assert_eq!(retrieved.as_ref().map(|b| b.len()), Some(size));
        }
        
        // DEL
        for i in 0..num_keys {
            let key = format!("stress:large:{}:{}", size, i);
            client.del(&key).await.expect("DEL failed");
        }
        
        let duration = start.elapsed();
        let total_bytes = (size * num_keys * 2) as f64; // SET + GET
        let throughput_mbps = (total_bytes / duration.as_secs_f64()) / (1024.0 * 1024.0);
        
        println!("  {} keys in {:?}", num_keys, duration);
        println!("  Throughput: {:.2} MB/s", throughput_mbps);
    }
}
