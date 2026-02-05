use bytes::Bytes;
use muxis::Client;

#[tokio::test]
#[ignore]
async fn test_sadd_and_srem() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.del("myset").await.ok();

    let added = client.sadd("myset", &[Bytes::from("a"), Bytes::from("b"), Bytes::from("c")]).await.unwrap();
    assert_eq!(added, 3);

    let added_again = client.sadd("myset", &[Bytes::from("a"), Bytes::from("d")]).await.unwrap();
    assert_eq!(added_again, 1);

    let removed = client.srem("myset", &[Bytes::from("a"), Bytes::from("b")]).await.unwrap();
    assert_eq!(removed, 2);
}

#[tokio::test]
#[ignore]
async fn test_smembers() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.del("memberset").await.ok();
    client.sadd("memberset", &[Bytes::from("a"), Bytes::from("b"), Bytes::from("c")]).await.unwrap();

    let members = client.smembers("memberset").await.unwrap();
    assert_eq!(members.len(), 3);
    assert!(members.contains(&"a".to_string()));
    assert!(members.contains(&"b".to_string()));
    assert!(members.contains(&"c".to_string()));
}

#[tokio::test]
#[ignore]
async fn test_sismember() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.del("ismemberset").await.ok();
    client.sadd("ismemberset", &[Bytes::from("a"), Bytes::from("b")]).await.unwrap();

    let is_member = client.sismember("ismemberset", Bytes::from("a")).await.unwrap();
    assert!(is_member);

    let not_member = client.sismember("ismemberset", Bytes::from("c")).await.unwrap();
    assert!(!not_member);
}

#[tokio::test]
#[ignore]
async fn test_scard() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.del("cardset").await.ok();

    let size = client.scard("cardset").await.unwrap();
    assert_eq!(size, 0);

    client.sadd("cardset", &[Bytes::from("a"), Bytes::from("b"), Bytes::from("c")]).await.unwrap();

    let size2 = client.scard("cardset").await.unwrap();
    assert_eq!(size2, 3);
}

#[tokio::test]
#[ignore]
async fn test_spop() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.del("popset").await.ok();
    client.sadd("popset", &[Bytes::from("a"), Bytes::from("b"), Bytes::from("c")]).await.unwrap();

    let popped = client.spop("popset").await.unwrap();
    assert!(popped.is_some());

    let remaining = client.scard("popset").await.unwrap();
    assert_eq!(remaining, 2);
}

#[tokio::test]
#[ignore]
async fn test_srandmember() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.del("randset").await.ok();
    client.sadd("randset", &[Bytes::from("a"), Bytes::from("b"), Bytes::from("c")]).await.unwrap();

    let random = client.srandmember("randset").await.unwrap();
    assert!(random.is_some());

    let size = client.scard("randset").await.unwrap();
    assert_eq!(size, 3);
}

#[tokio::test]
#[ignore]
async fn test_sdiff() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.del("set1").await.ok();
    client.del("set2").await.ok();

    client.sadd("set1", &[Bytes::from("a"), Bytes::from("b"), Bytes::from("c")]).await.unwrap();
    client.sadd("set2", &[Bytes::from("b"), Bytes::from("c"), Bytes::from("d")]).await.unwrap();

    let diff = client.sdiff(&["set1", "set2"]).await.unwrap();
    assert_eq!(diff.len(), 1);
    assert!(diff.contains(&"a".to_string()));
}

#[tokio::test]
#[ignore]
async fn test_sinter() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.del("set3").await.ok();
    client.del("set4").await.ok();

    client.sadd("set3", &[Bytes::from("a"), Bytes::from("b"), Bytes::from("c")]).await.unwrap();
    client.sadd("set4", &[Bytes::from("b"), Bytes::from("c"), Bytes::from("d")]).await.unwrap();

    let inter = client.sinter(&["set3", "set4"]).await.unwrap();
    assert_eq!(inter.len(), 2);
    assert!(inter.contains(&"b".to_string()));
    assert!(inter.contains(&"c".to_string()));
}

#[tokio::test]
#[ignore]
async fn test_sunion() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.del("set5").await.ok();
    client.del("set6").await.ok();

    client.sadd("set5", &[Bytes::from("a"), Bytes::from("b")]).await.unwrap();
    client.sadd("set6", &[Bytes::from("b"), Bytes::from("c")]).await.unwrap();

    let union = client.sunion(&["set5", "set6"]).await.unwrap();
    assert_eq!(union.len(), 3);
    assert!(union.contains(&"a".to_string()));
    assert!(union.contains(&"b".to_string()));
    assert!(union.contains(&"c".to_string()));
}

#[tokio::test]
#[ignore]
async fn test_sdiffstore() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.del("diffsrc1").await.ok();
    client.del("diffsrc2").await.ok();
    client.del("diffdest").await.ok();

    client.sadd("diffsrc1", &[Bytes::from("a"), Bytes::from("b"), Bytes::from("c")]).await.unwrap();
    client.sadd("diffsrc2", &[Bytes::from("b"), Bytes::from("c")]).await.unwrap();

    let count = client.sdiffstore("diffdest", &["diffsrc1", "diffsrc2"]).await.unwrap();
    assert_eq!(count, 1);

    let members = client.smembers("diffdest").await.unwrap();
    assert_eq!(members.len(), 1);
    assert!(members.contains(&"a".to_string()));
}

#[tokio::test]
#[ignore]
async fn test_sinterstore() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.del("intersrc1").await.ok();
    client.del("intersrc2").await.ok();
    client.del("interdest").await.ok();

    client.sadd("intersrc1", &[Bytes::from("a"), Bytes::from("b"), Bytes::from("c")]).await.unwrap();
    client.sadd("intersrc2", &[Bytes::from("b"), Bytes::from("c"), Bytes::from("d")]).await.unwrap();

    let count = client.sinterstore("interdest", &["intersrc1", "intersrc2"]).await.unwrap();
    assert_eq!(count, 2);

    let members = client.smembers("interdest").await.unwrap();
    assert_eq!(members.len(), 2);
    assert!(members.contains(&"b".to_string()));
    assert!(members.contains(&"c".to_string()));
}

#[tokio::test]
#[ignore]
async fn test_sunionstore() {
    let mut client = Client::connect("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect");

    client.del("unionsrc1").await.ok();
    client.del("unionsrc2").await.ok();
    client.del("uniondest").await.ok();

    client.sadd("unionsrc1", &[Bytes::from("a"), Bytes::from("b")]).await.unwrap();
    client.sadd("unionsrc2", &[Bytes::from("b"), Bytes::from("c")]).await.unwrap();

    let count = client.sunionstore("uniondest", &["unionsrc1", "unionsrc2"]).await.unwrap();
    assert_eq!(count, 3);

    let members = client.smembers("uniondest").await.unwrap();
    assert_eq!(members.len(), 3);
}
