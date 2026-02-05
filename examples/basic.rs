//! Basic usage example for Muxis Redis client.
//!
//! Run with: cargo run --example basic

use bytes::Bytes;
use muxis::{Client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to Redis server
    let mut client = Client::connect("redis://127.0.0.1:6379").await?;

    println!("Connected to Redis!");

    // Ping the server
    let pong = client.ping().await?;
    println!("PING response: {:?}", pong);

    // Set a key
    client.set("mykey", Bytes::from("Hello, Muxis!")).await?;
    println!("SET mykey");

    // Get the key
    if let Some(value) = client.get("mykey").await? {
        let s = String::from_utf8_lossy(&value);
        println!("GET mykey: {}", s);
    }

    // Increment a counter
    let count = client.incr("counter").await?;
    println!("INCR counter: {}", count);

    // Delete keys
    let deleted = client.del("mykey").await?;
    println!("DEL mykey: {}", deleted);

    let deleted = client.del("counter").await?;
    println!("DEL counter: {}", deleted);

    Ok(())
}
