//! Example using ClientBuilder for advanced configuration.
//!
//! Run with: cargo run --example builder

use bytes::Bytes;
use muxis::{ClientBuilder, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Build a client with custom configuration
    let mut client = ClientBuilder::new()
        .address("redis://127.0.0.1:6379")
        .database(0)
        .build()
        .await?;

    println!("Connected to Redis with custom configuration!");

    // Test connection
    let pong = client.ping().await?;
    println!("PING response: {:?}", pong);

    // Use the client
    client
        .set("builder_example", Bytes::from("Built with ClientBuilder"))
        .await?;

    if let Some(value) = client.get("builder_example").await? {
        println!("Value: {}", String::from_utf8_lossy(&value));
    }

    // Cleanup
    client.del("builder_example").await?;

    Ok(())
}
