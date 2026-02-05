//! Example demonstrating multiple commands in sequence.
//!
//! Run with: cargo run --example pipeline

use bytes::Bytes;
use muxis::{Client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Client::connect("redis://127.0.0.1:6379").await?;

    println!("Connected to Redis!");

    // Execute multiple commands
    println!("\nSetting multiple keys...");
    client.set("user:1:name", Bytes::from("Alice")).await?;
    client.set("user:1:age", Bytes::from("30")).await?;
    client.set("user:1:city", Bytes::from("New York")).await?;

    // Retrieve all values
    println!("\nRetrieving user data...");
    if let Some(name) = client.get("user:1:name").await? {
        println!("Name: {}", String::from_utf8_lossy(&name));
    }

    if let Some(age) = client.get("user:1:age").await? {
        println!("Age: {}", String::from_utf8_lossy(&age));
    }

    if let Some(city) = client.get("user:1:city").await? {
        println!("City: {}", String::from_utf8_lossy(&city));
    }

    // Increment a counter multiple times
    println!("\nIncrementing page views...");
    for _ in 0..5 {
        let views = client.incr("page:views").await?;
        println!("Page views: {}", views);
    }

    // Cleanup
    println!("\nCleaning up...");
    let mut deleted_count = 0;
    if client.del("user:1:name").await? {
        deleted_count += 1;
    }
    if client.del("user:1:age").await? {
        deleted_count += 1;
    }
    if client.del("user:1:city").await? {
        deleted_count += 1;
    }
    if client.del("page:views").await? {
        deleted_count += 1;
    }
    println!("Deleted {} keys", deleted_count);

    Ok(())
}
