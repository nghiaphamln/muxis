use bytes::Bytes;
use muxis::Client;

#[tokio::test]
#[ignore]
async fn test_lpush_and_rpush() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.del("mylist").await.ok();

    let len1 = client.lpush("mylist", &[Bytes::from("a")]).await.unwrap();
    assert_eq!(len1, 1);

    let len2 = client
        .rpush("mylist", &[Bytes::from("b"), Bytes::from("c")])
        .await
        .unwrap();
    assert_eq!(len2, 3);
}

#[tokio::test]
#[ignore]
async fn test_lpop_and_rpop() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.del("poplist").await.ok();
    client
        .rpush(
            "poplist",
            &[Bytes::from("a"), Bytes::from("b"), Bytes::from("c")],
        )
        .await
        .unwrap();

    let first = client.lpop("poplist").await.unwrap();
    assert_eq!(first, Some(Bytes::from("a")));

    let last = client.rpop("poplist").await.unwrap();
    assert_eq!(last, Some(Bytes::from("c")));

    let len = client.llen("poplist").await.unwrap();
    assert_eq!(len, 1);
}

#[tokio::test]
#[ignore]
async fn test_llen() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.del("lenlist").await.ok();

    let len = client.llen("lenlist").await.unwrap();
    assert_eq!(len, 0);

    client
        .rpush("lenlist", &[Bytes::from("a"), Bytes::from("b")])
        .await
        .unwrap();
    let len2 = client.llen("lenlist").await.unwrap();
    assert_eq!(len2, 2);
}

#[tokio::test]
#[ignore]
async fn test_lrange() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.del("rangelist").await.ok();
    client
        .rpush(
            "rangelist",
            &[
                Bytes::from("a"),
                Bytes::from("b"),
                Bytes::from("c"),
                Bytes::from("d"),
            ],
        )
        .await
        .unwrap();

    let range = client.lrange("rangelist", 1, 2).await.unwrap();
    assert_eq!(range.len(), 2);
    assert_eq!(range[0], Bytes::from("b"));
    assert_eq!(range[1], Bytes::from("c"));

    let all = client.lrange("rangelist", 0, -1).await.unwrap();
    assert_eq!(all.len(), 4);
}

#[tokio::test]
#[ignore]
async fn test_lindex() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.del("indexlist").await.ok();
    client
        .rpush(
            "indexlist",
            &[Bytes::from("a"), Bytes::from("b"), Bytes::from("c")],
        )
        .await
        .unwrap();

    let elem = client.lindex("indexlist", 1).await.unwrap();
    assert_eq!(elem, Some(Bytes::from("b")));

    let last = client.lindex("indexlist", -1).await.unwrap();
    assert_eq!(last, Some(Bytes::from("c")));

    let invalid = client.lindex("indexlist", 10).await.unwrap();
    assert_eq!(invalid, None);
}

#[tokio::test]
#[ignore]
async fn test_lset() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.del("setlist").await.ok();
    client
        .rpush(
            "setlist",
            &[Bytes::from("a"), Bytes::from("b"), Bytes::from("c")],
        )
        .await
        .unwrap();

    client.lset("setlist", 1, Bytes::from("x")).await.unwrap();

    let elem = client.lindex("setlist", 1).await.unwrap();
    assert_eq!(elem, Some(Bytes::from("x")));
}

#[tokio::test]
#[ignore]
async fn test_lrem() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.del("remlist").await.ok();
    client
        .rpush(
            "remlist",
            &[
                Bytes::from("a"),
                Bytes::from("b"),
                Bytes::from("a"),
                Bytes::from("c"),
                Bytes::from("a"),
            ],
        )
        .await
        .unwrap();

    let removed = client.lrem("remlist", 2, Bytes::from("a")).await.unwrap();
    assert_eq!(removed, 2);

    let remaining = client.lrange("remlist", 0, -1).await.unwrap();
    assert_eq!(remaining.len(), 3);
}

#[tokio::test]
#[ignore]
async fn test_ltrim() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.del("trimlist").await.ok();
    client
        .rpush(
            "trimlist",
            &[
                Bytes::from("a"),
                Bytes::from("b"),
                Bytes::from("c"),
                Bytes::from("d"),
                Bytes::from("e"),
            ],
        )
        .await
        .unwrap();

    client.ltrim("trimlist", 1, 3).await.unwrap();

    let remaining = client.lrange("trimlist", 0, -1).await.unwrap();
    assert_eq!(remaining.len(), 3);
    assert_eq!(remaining[0], Bytes::from("b"));
    assert_eq!(remaining[2], Bytes::from("d"));
}

#[tokio::test]
#[ignore]
async fn test_rpoplpush() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.del("source").await.ok();
    client.del("dest").await.ok();

    client
        .rpush(
            "source",
            &[Bytes::from("a"), Bytes::from("b"), Bytes::from("c")],
        )
        .await
        .unwrap();

    let elem = client.rpoplpush("source", "dest").await.unwrap();
    assert_eq!(elem, Some(Bytes::from("c")));

    let source_len = client.llen("source").await.unwrap();
    assert_eq!(source_len, 2);

    let dest_len = client.llen("dest").await.unwrap();
    assert_eq!(dest_len, 1);
}

#[tokio::test]
#[ignore]
async fn test_blpop_timeout() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.del("blpoplist").await.ok();

    let result = client.blpop(&["blpoplist"], 1).await.unwrap();
    assert_eq!(result, None);
}

#[tokio::test]
#[ignore]
async fn test_blpop_immediate() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.del("blpopimmediate").await.ok();
    client
        .rpush("blpopimmediate", &[Bytes::from("value")])
        .await
        .unwrap();

    let result = client.blpop(&["blpopimmediate"], 5).await.unwrap();
    assert!(result.is_some());
    let (key, value) = result.unwrap();
    assert_eq!(key, "blpopimmediate");
    assert_eq!(value, Bytes::from("value"));
}

#[tokio::test]
#[ignore]
async fn test_lpos() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.del("poslist").await.ok();
    client
        .rpush(
            "poslist",
            &[
                Bytes::from("a"),
                Bytes::from("b"),
                Bytes::from("c"),
                Bytes::from("b"),
            ],
        )
        .await
        .unwrap();

    let pos = client.lpos("poslist", Bytes::from("b")).await.unwrap();
    assert_eq!(pos, Some(1));

    let not_found = client.lpos("poslist", Bytes::from("x")).await.unwrap();
    assert_eq!(not_found, None);
}
