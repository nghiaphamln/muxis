//! Benchmarks for Redis Cluster operations.
//!
//! These benchmarks require a real Redis Cluster running on localhost.
//!
//! Setup with Docker:
//! ```bash
//! docker run -d --name redis-cluster \
//!   -p 7000-7005:7000-7005 \
//!   grokzen/redis-cluster:latest
//! ```
//!
//! Run benchmarks:
//! ```bash
//! cargo bench --bench cluster_benchmark --features cluster
//! ```

use bytes::Bytes;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use muxis::ClusterClient;
use tokio::runtime::Runtime;

/// Creates a test client for benchmarking.
fn create_client() -> ClusterClient {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        ClusterClient::connect("127.0.0.1:7000,127.0.0.1:7001,127.0.0.1:7002")
            .await
            .expect("failed to connect to cluster")
    })
}

/// Benchmark: SET operation with different value sizes.
fn bench_cluster_set(c: &mut Criterion) {
    let mut group = c.benchmark_group("cluster_set");
    let rt = Runtime::new().unwrap();
    let client = create_client();

    for size in [64, 256, 1024, 4096, 16384].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let value = Bytes::from(vec![b'x'; size]);
            let key = format!("bench:set:{}", size);

            b.to_async(&rt).iter(|| async {
                client
                    .set(black_box(&key), black_box(value.clone()))
                    .await
                    .expect("SET failed");
            });
        });
    }

    group.finish();
}

/// Benchmark: GET operation with different value sizes.
fn bench_cluster_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("cluster_get");
    let rt = Runtime::new().unwrap();
    let client = create_client();

    // Prepare data
    for size in [64, 256, 1024, 4096, 16384].iter() {
        let value = Bytes::from(vec![b'x'; *size]);
        let key = format!("bench:get:{}", size);
        rt.block_on(async {
            client
                .set(&key, value)
                .await
                .expect("failed to prepare data");
        });
    }

    for size in [64, 256, 1024, 4096, 16384].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let key = format!("bench:get:{}", size);

            b.to_async(&rt).iter(|| async {
                client.get(black_box(&key)).await.expect("GET failed");
            });
        });
    }

    group.finish();
}

/// Benchmark: DEL operation.
fn bench_cluster_del(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let client = create_client();

    c.bench_function("cluster_del", |b| {
        b.to_async(&rt).iter(|| async {
            let key = "bench:del:test";

            // SET before each DEL
            client
                .set(key, Bytes::from("value"))
                .await
                .expect("SET failed");

            // Benchmark the DEL
            client.del(black_box(key)).await.expect("DEL failed");
        });
    });
}

/// Benchmark: EXISTS operation.
fn bench_cluster_exists(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let client = create_client();

    // Prepare data
    rt.block_on(async {
        client
            .set("bench:exists:test", Bytes::from("value"))
            .await
            .expect("failed to prepare data");
    });

    c.bench_function("cluster_exists", |b| {
        let key = "bench:exists:test";

        b.to_async(&rt).iter(|| async {
            client.exists(black_box(key)).await.expect("EXISTS failed");
        });
    });
}

/// Benchmark: Slot calculation.
fn bench_slot_calculation(c: &mut Criterion) {
    use muxis::key_slot;

    let mut group = c.benchmark_group("slot_calculation");

    // Short key
    group.bench_function("short_key", |b| {
        b.iter(|| key_slot(black_box("key")));
    });

    // Long key
    group.bench_function("long_key", |b| {
        let key = "a".repeat(100);
        b.iter(|| key_slot(black_box(&key)));
    });

    // Key with hash tag
    group.bench_function("hash_tag", |b| {
        b.iter(|| key_slot(black_box("user:{12345}:profile")));
    });

    group.finish();
}

/// Benchmark: Multi-key validation.
fn bench_validate_same_slot(c: &mut Criterion) {
    let mut group = c.benchmark_group("validate_same_slot");

    // 2 keys
    group.bench_function("2_keys", |b| {
        let keys = vec!["{tag}:key1", "{tag}:key2"];
        b.iter(|| ClusterClient::validate_same_slot(black_box(&keys)));
    });

    // 5 keys
    group.bench_function("5_keys", |b| {
        let keys = vec![
            "{tag}:key1",
            "{tag}:key2",
            "{tag}:key3",
            "{tag}:key4",
            "{tag}:key5",
        ];
        b.iter(|| ClusterClient::validate_same_slot(black_box(&keys)));
    });

    // 10 keys
    group.bench_function("10_keys", |b| {
        let keys = vec![
            "{tag}:key1",
            "{tag}:key2",
            "{tag}:key3",
            "{tag}:key4",
            "{tag}:key5",
            "{tag}:key6",
            "{tag}:key7",
            "{tag}:key8",
            "{tag}:key9",
            "{tag}:key10",
        ];
        b.iter(|| ClusterClient::validate_same_slot(black_box(&keys)));
    });

    group.finish();
}

/// Benchmark: Concurrent operations.
fn bench_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");
    let rt = Runtime::new().unwrap();
    let client = create_client();

    for num_tasks in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_tasks),
            num_tasks,
            |b, &num_tasks| {
                b.to_async(&rt).iter(|| async {
                    let mut handles = vec![];

                    for i in 0..num_tasks {
                        let client_clone = client.clone();
                        let handle = tokio::spawn(async move {
                            let key = format!("bench:concurrent:{}", i);
                            let value = Bytes::from("value");

                            client_clone.set(&key, value).await?;
                            client_clone.get(&key).await?;
                            client_clone.del(&key).await?;

                            Ok::<_, muxis::Error>(())
                        });
                        handles.push(handle);
                    }

                    for handle in handles {
                        handle.await.unwrap().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Topology operations.
fn bench_topology_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let client = create_client();

    c.bench_function("node_count", |b| {
        b.to_async(&rt).iter(|| async { client.node_count().await });
    });

    c.bench_function("is_fully_covered", |b| {
        b.to_async(&rt)
            .iter(|| async { client.is_fully_covered().await });
    });

    c.bench_function("refresh_topology", |b| {
        b.to_async(&rt).iter(|| async {
            client.refresh_topology().await.expect("refresh failed");
        });
    });
}

criterion_group!(
    benches,
    bench_cluster_set,
    bench_cluster_get,
    bench_cluster_del,
    bench_cluster_exists,
    bench_slot_calculation,
    bench_validate_same_slot,
    bench_concurrent_operations,
    bench_topology_operations
);

criterion_main!(benches);
