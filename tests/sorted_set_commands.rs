//! Integration tests for Redis Sorted Set commands.
//!
//! Run with: cargo test --test sorted_set_commands -- --ignored
//! Requires Redis running at 127.0.0.1:6379

use bytes::Bytes;
use muxis::Client;

#[tokio::test]
#[ignore]
async fn test_zadd_and_zrange() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    let key = "test:zset:zadd";
    let _ = client.del(key).await;

    let count = client
        .zadd(
            key,
            &[
                (1.0, Bytes::from("one")),
                (2.0, Bytes::from("two")),
                (3.0, Bytes::from("three")),
            ],
        )
        .await
        .expect("ZADD failed");
    assert_eq!(count, 3);

    let members = client.zrange(key, 0, -1).await.expect("ZRANGE failed");
    assert_eq!(members, vec!["one", "two", "three"]);

    let _ = client.del(key).await;
}

#[tokio::test]
#[ignore]
async fn test_zrem() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    let key = "test:zset:zrem";
    let _ = client.del(key).await;

    client
        .zadd(
            key,
            &[
                (1.0, Bytes::from("a")),
                (2.0, Bytes::from("b")),
                (3.0, Bytes::from("c")),
            ],
        )
        .await
        .unwrap();

    let removed = client
        .zrem(key, &[Bytes::from("b")])
        .await
        .expect("ZREM failed");
    assert_eq!(removed, 1);

    let members = client.zrange(key, 0, -1).await.unwrap();
    assert_eq!(members, vec!["a", "c"]);

    let _ = client.del(key).await;
}

#[tokio::test]
#[ignore]
async fn test_zrank_and_zscore() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    let key = "test:zset:rank";
    let _ = client.del(key).await;

    client
        .zadd(
            key,
            &[
                (10.0, Bytes::from("a")),
                (20.0, Bytes::from("b")),
                (30.0, Bytes::from("c")),
            ],
        )
        .await
        .unwrap();

    let rank = client
        .zrank(key, Bytes::from("b"))
        .await
        .expect("ZRANK failed");
    assert_eq!(rank, Some(1));

    let score = client
        .zscore(key, Bytes::from("b"))
        .await
        .expect("ZSCORE failed");
    assert_eq!(score, Some(20.0));

    let _ = client.del(key).await;
}

#[tokio::test]
#[ignore]
async fn test_zcard() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    let key = "test:zset:zcard";
    let _ = client.del(key).await;

    client
        .zadd(
            key,
            &[
                (1.0, Bytes::from("a")),
                (2.0, Bytes::from("b")),
                (3.0, Bytes::from("c")),
            ],
        )
        .await
        .unwrap();

    let card = client.zcard(key).await.expect("ZCARD failed");
    assert_eq!(card, 3);

    let _ = client.del(key).await;
}

#[tokio::test]
#[ignore]
async fn test_zcount() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    let key = "test:zset:zcount";
    let _ = client.del(key).await;

    client
        .zadd(
            key,
            &[
                (1.0, Bytes::from("a")),
                (2.0, Bytes::from("b")),
                (3.0, Bytes::from("c")),
                (4.0, Bytes::from("d")),
            ],
        )
        .await
        .unwrap();

    let count = client.zcount(key, "2", "3").await.expect("ZCOUNT failed");
    assert_eq!(count, 2);

    let _ = client.del(key).await;
}

#[tokio::test]
#[ignore]
async fn test_zincrby() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    let key = "test:zset:zincrby";
    let _ = client.del(key).await;

    client
        .zadd(key, &[(10.0, Bytes::from("member"))])
        .await
        .unwrap();

    let new_score = client
        .zincrby(key, 5.5, Bytes::from("member"))
        .await
        .expect("ZINCRBY failed");
    assert_eq!(new_score, 15.5);

    let _ = client.del(key).await;
}

#[tokio::test]
#[ignore]
async fn test_zrevrange() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    let key = "test:zset:zrevrange";
    let _ = client.del(key).await;

    client
        .zadd(
            key,
            &[
                (1.0, Bytes::from("one")),
                (2.0, Bytes::from("two")),
                (3.0, Bytes::from("three")),
            ],
        )
        .await
        .unwrap();

    let members = client
        .zrevrange(key, 0, -1)
        .await
        .expect("ZREVRANGE failed");
    assert_eq!(members, vec!["three", "two", "one"]);

    let _ = client.del(key).await;
}

#[tokio::test]
#[ignore]
async fn test_zrevrank() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    let key = "test:zset:zrevrank";
    let _ = client.del(key).await;

    client
        .zadd(
            key,
            &[
                (10.0, Bytes::from("a")),
                (20.0, Bytes::from("b")),
                (30.0, Bytes::from("c")),
            ],
        )
        .await
        .unwrap();

    let rank = client
        .zrevrank(key, Bytes::from("b"))
        .await
        .expect("ZREVRANK failed");
    assert_eq!(rank, Some(1));

    let _ = client.del(key).await;
}

#[tokio::test]
#[ignore]
async fn test_zrangebyscore() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    let key = "test:zset:zrangebyscore";
    let _ = client.del(key).await;

    client
        .zadd(
            key,
            &[
                (1.0, Bytes::from("a")),
                (2.0, Bytes::from("b")),
                (3.0, Bytes::from("c")),
                (4.0, Bytes::from("d")),
            ],
        )
        .await
        .unwrap();

    let members = client
        .zrangebyscore(key, "2", "3")
        .await
        .expect("ZRANGEBYSCORE failed");
    assert_eq!(members, vec!["b", "c"]);

    let _ = client.del(key).await;
}

#[tokio::test]
#[ignore]
async fn test_zremrangebyrank() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    let key = "test:zset:zremrangebyrank";
    let _ = client.del(key).await;

    client
        .zadd(
            key,
            &[
                (1.0, Bytes::from("a")),
                (2.0, Bytes::from("b")),
                (3.0, Bytes::from("c")),
            ],
        )
        .await
        .unwrap();

    let removed = client
        .zremrangebyrank(key, 0, 1)
        .await
        .expect("ZREMRANGEBYRANK failed");
    assert_eq!(removed, 2);

    let members = client.zrange(key, 0, -1).await.unwrap();
    assert_eq!(members, vec!["c"]);

    let _ = client.del(key).await;
}

#[tokio::test]
#[ignore]
async fn test_zremrangebyscore() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    let key = "test:zset:zremrangebyscore";
    let _ = client.del(key).await;

    client
        .zadd(
            key,
            &[
                (1.0, Bytes::from("a")),
                (2.0, Bytes::from("b")),
                (3.0, Bytes::from("c")),
                (4.0, Bytes::from("d")),
            ],
        )
        .await
        .unwrap();

    let removed = client
        .zremrangebyscore(key, "2", "3")
        .await
        .expect("ZREMRANGEBYSCORE failed");
    assert_eq!(removed, 2);

    let members = client.zrange(key, 0, -1).await.unwrap();
    assert_eq!(members, vec!["a", "d"]);

    let _ = client.del(key).await;
}

#[tokio::test]
#[ignore]
async fn test_zpopmin() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    let key = "test:zset:zpopmin";
    let _ = client.del(key).await;

    client
        .zadd(
            key,
            &[
                (1.0, Bytes::from("a")),
                (2.0, Bytes::from("b")),
                (3.0, Bytes::from("c")),
            ],
        )
        .await
        .unwrap();

    let result = client.zpopmin(key).await.expect("ZPOPMIN failed");
    assert!(result.is_some());
    let (member, score) = result.unwrap();
    assert_eq!(member, "a");
    assert_eq!(score, 1.0);

    let _ = client.del(key).await;
}

#[tokio::test]
#[ignore]
async fn test_zpopmax() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    let key = "test:zset:zpopmax";
    let _ = client.del(key).await;

    client
        .zadd(
            key,
            &[
                (1.0, Bytes::from("a")),
                (2.0, Bytes::from("b")),
                (3.0, Bytes::from("c")),
            ],
        )
        .await
        .unwrap();

    let result = client.zpopmax(key).await.expect("ZPOPMAX failed");
    assert!(result.is_some());
    let (member, score) = result.unwrap();
    assert_eq!(member, "c");
    assert_eq!(score, 3.0);

    let _ = client.del(key).await;
}

#[tokio::test]
#[ignore]
async fn test_zlexcount() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    let key = "test:zset:zlexcount";
    let _ = client.del(key).await;

    client
        .zadd(
            key,
            &[
                (0.0, Bytes::from("a")),
                (0.0, Bytes::from("b")),
                (0.0, Bytes::from("c")),
                (0.0, Bytes::from("d")),
            ],
        )
        .await
        .unwrap();

    let count = client
        .zlexcount(key, "[b", "[c")
        .await
        .expect("ZLEXCOUNT failed");
    assert_eq!(count, 2);

    let _ = client.del(key).await;
}

#[tokio::test]
#[ignore]
async fn test_zrangebylex() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    let key = "test:zset:zrangebylex";
    let _ = client.del(key).await;

    client
        .zadd(
            key,
            &[
                (0.0, Bytes::from("a")),
                (0.0, Bytes::from("b")),
                (0.0, Bytes::from("c")),
                (0.0, Bytes::from("d")),
            ],
        )
        .await
        .unwrap();

    let members = client
        .zrangebylex(key, "[b", "[c")
        .await
        .expect("ZRANGEBYLEX failed");
    assert_eq!(members, vec!["b", "c"]);

    let _ = client.del(key).await;
}

#[tokio::test]
#[ignore]
async fn test_zremrangebylex() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    let key = "test:zset:zremrangebylex";
    let _ = client.del(key).await;

    client
        .zadd(
            key,
            &[
                (0.0, Bytes::from("a")),
                (0.0, Bytes::from("b")),
                (0.0, Bytes::from("c")),
                (0.0, Bytes::from("d")),
            ],
        )
        .await
        .unwrap();

    let removed = client
        .zremrangebylex(key, "[b", "[c")
        .await
        .expect("ZREMRANGEBYLEX failed");
    assert_eq!(removed, 2);

    let members = client.zrange(key, 0, -1).await.unwrap();
    assert_eq!(members, vec!["a", "d"]);

    let _ = client.del(key).await;
}
