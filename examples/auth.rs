//! Example demonstrating authentication with Redis.
//!
//! Run with: cargo run --example auth
//!
//! Note: This requires a Redis server configured with authentication.
//! Set password in redis.conf: requirepass yourpassword

use bytes::Bytes;
use muxis::{Client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Method 1: Connect with password in URL
    let url = "redis://:mypassword@127.0.0.1:6379";
    let mut client = Client::connect(url).await?;

    println!("Connected to Redis with authentication!");

    // Test connection
    let pong = client.ping().await?;
    println!("PING response: {:?}", pong);

    // Method 2: Authenticate after connecting (if needed)
    // client.auth("mypassword").await?;

    // Use the authenticated client
    client
        .set("authenticated_key", Bytes::from("Secret value"))
        .await?;

    if let Some(value) = client.get("authenticated_key").await? {
        println!("Value: {}", String::from_utf8_lossy(&value));
    }

    // Select a different database
    client.select(1).await?;
    println!("Switched to database 1");

    client.set("db1_key", Bytes::from("Value in DB 1")).await?;

    // Switch back to database 0
    client.select(0).await?;
    println!("Switched back to database 0");

    // Cleanup
    client.del("authenticated_key").await?;

    Ok(())
}
