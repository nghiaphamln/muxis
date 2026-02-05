use bytes::Bytes;
use muxis::Client;

#[tokio::test]
#[ignore]
async fn test_hset_and_hget() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    let was_new = client
        .hset("myhash", "field1", Bytes::from("value1"))
        .await
        .unwrap();
    assert!(was_new);

    let value = client.hget("myhash", "field1").await.unwrap();
    assert_eq!(value, Some(Bytes::from("value1")));

    let was_new_again = client
        .hset("myhash", "field1", Bytes::from("value2"))
        .await
        .unwrap();
    assert!(!was_new_again);
}

#[tokio::test]
#[ignore]
async fn test_hmset_and_hmget() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client
        .hmset(
            "myhash2",
            &[
                ("field1", Bytes::from("value1")),
                ("field2", Bytes::from("value2")),
                ("field3", Bytes::from("value3")),
            ],
        )
        .await
        .unwrap();

    let values = client
        .hmget("myhash2", &["field1", "field2", "field4"])
        .await
        .unwrap();

    assert_eq!(values.len(), 3);
    assert_eq!(values[0], Some(Bytes::from("value1")));
    assert_eq!(values[1], Some(Bytes::from("value2")));
    assert_eq!(values[2], None);
}

#[tokio::test]
#[ignore]
async fn test_hgetall() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client
        .hmset(
            "myhash3",
            &[
                ("field1", Bytes::from("value1")),
                ("field2", Bytes::from("value2")),
            ],
        )
        .await
        .unwrap();

    let all = client.hgetall("myhash3").await.unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all.get("field1"), Some(&Bytes::from("value1")));
    assert_eq!(all.get("field2"), Some(&Bytes::from("value2")));
}

#[tokio::test]
#[ignore]
async fn test_hdel() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client
        .hmset(
            "myhash4",
            &[
                ("field1", Bytes::from("value1")),
                ("field2", Bytes::from("value2")),
                ("field3", Bytes::from("value3")),
            ],
        )
        .await
        .unwrap();

    let deleted = client.hdel("myhash4", &["field1", "field2"]).await.unwrap();
    assert_eq!(deleted, 2);

    let remaining = client.hgetall("myhash4").await.unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining.get("field3"), Some(&Bytes::from("value3")));
}

#[tokio::test]
#[ignore]
async fn test_hexists() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client
        .hset("myhash5", "field1", Bytes::from("value1"))
        .await
        .unwrap();

    let exists = client.hexists("myhash5", "field1").await.unwrap();
    assert!(exists);

    let not_exists = client.hexists("myhash5", "field2").await.unwrap();
    assert!(!not_exists);
}

#[tokio::test]
#[ignore]
async fn test_hlen() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client
        .hmset(
            "myhash6",
            &[
                ("field1", Bytes::from("value1")),
                ("field2", Bytes::from("value2")),
                ("field3", Bytes::from("value3")),
            ],
        )
        .await
        .unwrap();

    let len = client.hlen("myhash6").await.unwrap();
    assert_eq!(len, 3);
}

#[tokio::test]
#[ignore]
async fn test_hkeys_and_hvals() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client
        .hmset(
            "myhash7",
            &[
                ("field1", Bytes::from("value1")),
                ("field2", Bytes::from("value2")),
            ],
        )
        .await
        .unwrap();

    let keys = client.hkeys("myhash7").await.unwrap();
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&"field1".to_string()));
    assert!(keys.contains(&"field2".to_string()));

    let vals = client.hvals("myhash7").await.unwrap();
    assert_eq!(vals.len(), 2);
    assert!(vals.contains(&"value1".to_string()));
    assert!(vals.contains(&"value2".to_string()));
}

#[tokio::test]
#[ignore]
async fn test_hincrby() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client
        .hset("myhash8", "counter", Bytes::from("10"))
        .await
        .unwrap();

    let new_val = client.hincrby("myhash8", "counter", 5).await.unwrap();
    assert_eq!(new_val, 15);

    let new_val2 = client.hincrby("myhash8", "counter", -3).await.unwrap();
    assert_eq!(new_val2, 12);
}

#[tokio::test]
#[ignore]
async fn test_hincrbyfloat() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client
        .hset("myhash9", "price", Bytes::from("10.5"))
        .await
        .unwrap();

    let new_val = client.hincrbyfloat("myhash9", "price", 2.5).await.unwrap();
    assert!((new_val - 13.0).abs() < 0.001);
}

#[tokio::test]
#[ignore]
async fn test_hsetnx() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    let was_set = client
        .hsetnx("myhash10", "field1", Bytes::from("value1"))
        .await
        .unwrap();
    assert!(was_set);

    let was_set_again = client
        .hsetnx("myhash10", "field1", Bytes::from("value2"))
        .await
        .unwrap();
    assert!(!was_set_again);

    let value = client.hget("myhash10", "field1").await.unwrap();
    assert_eq!(value, Some(Bytes::from("value1")));
}
