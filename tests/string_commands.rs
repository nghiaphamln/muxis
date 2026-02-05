use bytes::Bytes;
use muxis::Client;

#[tokio::test]
#[ignore]
async fn test_mget() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.set("key1", Bytes::from("value1")).await.unwrap();
    client.set("key2", Bytes::from("value2")).await.unwrap();
    client.del("key3").await.ok();

    let values = client.mget(&["key1", "key2", "key3"]).await.unwrap();

    assert_eq!(values.len(), 3);
    assert_eq!(values[0], Some(Bytes::from("value1")));
    assert_eq!(values[1], Some(Bytes::from("value2")));
    assert_eq!(values[2], None);
}

#[tokio::test]
#[ignore]
async fn test_mset() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client
        .mset(&[
            ("msetkey1", Bytes::from("msetvalue1")),
            ("msetkey2", Bytes::from("msetvalue2")),
            ("msetkey3", Bytes::from("msetvalue3")),
        ])
        .await
        .unwrap();

    let val1 = client.get("msetkey1").await.unwrap();
    let val2 = client.get("msetkey2").await.unwrap();
    let val3 = client.get("msetkey3").await.unwrap();

    assert_eq!(val1, Some(Bytes::from("msetvalue1")));
    assert_eq!(val2, Some(Bytes::from("msetvalue2")));
    assert_eq!(val3, Some(Bytes::from("msetvalue3")));
}

#[tokio::test]
#[ignore]
async fn test_setnx() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.del("setnxkey").await.ok();

    let was_set = client
        .setnx("setnxkey", Bytes::from("setnxvalue"))
        .await
        .unwrap();
    assert!(was_set);

    let was_set_again = client
        .setnx("setnxkey", Bytes::from("newvalue"))
        .await
        .unwrap();
    assert!(!was_set_again);

    let value = client.get("setnxkey").await.unwrap();
    assert_eq!(value, Some(Bytes::from("setnxvalue")));
}

#[tokio::test]
#[ignore]
async fn test_setex() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client
        .setex("setexkey", 10, Bytes::from("setexvalue"))
        .await
        .unwrap();

    let value = client.get("setexkey").await.unwrap();
    assert_eq!(value, Some(Bytes::from("setexvalue")));

    tokio::time::sleep(tokio::time::Duration::from_secs(11)).await;

    let value_after = client.get("setexkey").await.unwrap();
    assert_eq!(value_after, None);
}

#[tokio::test]
#[ignore]
async fn test_getdel() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client
        .set("getdelkey", Bytes::from("getdelvalue"))
        .await
        .unwrap();

    let value = client.getdel("getdelkey").await.unwrap();
    assert_eq!(value, Some(Bytes::from("getdelvalue")));

    let value_after = client.get("getdelkey").await.unwrap();
    assert_eq!(value_after, None);

    let nonexistent = client.getdel("nonexistentkey").await.unwrap();
    assert_eq!(nonexistent, None);
}

#[tokio::test]
#[ignore]
async fn test_append() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.del("appendkey").await.ok();

    let len1 = client
        .append("appendkey", Bytes::from("Hello"))
        .await
        .unwrap();
    assert_eq!(len1, 5);

    let len2 = client
        .append("appendkey", Bytes::from(" World"))
        .await
        .unwrap();
    assert_eq!(len2, 11);

    let value = client.get("appendkey").await.unwrap();
    assert_eq!(value, Some(Bytes::from("Hello World")));
}

#[tokio::test]
#[ignore]
async fn test_strlen() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client
        .set("strlenkey", Bytes::from("Hello World"))
        .await
        .unwrap();

    let len = client.strlen("strlenkey").await.unwrap();
    assert_eq!(len, 11);

    let len_empty = client.strlen("nonexistentkey").await.unwrap();
    assert_eq!(len_empty, 0);
}
