use bytes::Bytes;
use muxis::Client;

#[tokio::test]
#[ignore]
async fn test_exists() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.set("existkey1", Bytes::from("value1")).await.unwrap();
    client.set("existkey2", Bytes::from("value2")).await.unwrap();
    client.del("existkey3").await.ok();

    let count = client
        .exists(&["existkey1", "existkey2", "existkey3"])
        .await
        .unwrap();
    assert_eq!(count, 2);
}

#[tokio::test]
#[ignore]
async fn test_key_type() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client
        .set("typekey", Bytes::from("value"))
        .await
        .unwrap();

    let key_type = client.key_type("typekey").await.unwrap();
    assert_eq!(key_type, "string");

    let nonexistent_type = client.key_type("nonexistentkey").await.unwrap();
    assert_eq!(nonexistent_type, "none");
}

#[tokio::test]
#[ignore]
async fn test_expire_and_ttl() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client
        .set("expirekey", Bytes::from("value"))
        .await
        .unwrap();

    let was_set = client.expire("expirekey", 60).await.unwrap();
    assert!(was_set);

    let ttl = client.ttl("expirekey").await.unwrap();
    assert!(ttl > 0 && ttl <= 60);

    let ttl_nonexistent = client.ttl("nonexistentkey").await.unwrap();
    assert_eq!(ttl_nonexistent, -2);
}

#[tokio::test]
#[ignore]
async fn test_expireat() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client
        .set("expireatkey", Bytes::from("value"))
        .await
        .unwrap();

    let future_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 60;

    let was_set = client.expireat("expireatkey", future_timestamp).await.unwrap();
    assert!(was_set);

    let ttl = client.ttl("expireatkey").await.unwrap();
    assert!(ttl > 0 && ttl <= 60);
}

#[tokio::test]
#[ignore]
async fn test_persist() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client
        .setex("persistkey", 60, Bytes::from("value"))
        .await
        .unwrap();

    let was_persisted = client.persist("persistkey").await.unwrap();
    assert!(was_persisted);

    let ttl = client.ttl("persistkey").await.unwrap();
    assert_eq!(ttl, -1);
}

#[tokio::test]
#[ignore]
async fn test_rename() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client
        .set("renameoldkey", Bytes::from("value"))
        .await
        .unwrap();

    client.rename("renameoldkey", "renamenewkey").await.unwrap();

    let value = client.get("renamenewkey").await.unwrap();
    assert_eq!(value, Some(Bytes::from("value")));

    let old_value = client.get("renameoldkey").await.unwrap();
    assert_eq!(old_value, None);
}

#[tokio::test]
#[ignore]
async fn test_scan() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    for i in 0..10 {
        client
            .set(&format!("scankey{}", i), Bytes::from("value"))
            .await
            .unwrap();
    }

    let mut all_keys = Vec::new();
    let mut cursor = 0;

    loop {
        let (next_cursor, keys) = client.scan(cursor).await.unwrap();
        all_keys.extend(keys);
        cursor = next_cursor;
        if cursor == 0 {
            break;
        }
    }

    let scan_keys_count = all_keys.iter().filter(|k| k.starts_with("scankey")).count();
    assert!(scan_keys_count >= 10);
}
