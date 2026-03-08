//! Example: DHT peer discovery
//!
//! This example shows how to initialize a DHT manager,
//! store data, and look up peers by node ID.
//!
//! Run with:
//!   cargo run --example dht_lookup

use aamn::dht::{DhtManager, NodeId};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Bootstrap peers (in production, these come from config)
    let bootstrap: Vec<SocketAddr> =
        vec!["203.0.113.5:9000".parse()?, "198.51.100.8:9000".parse()?];

    // Create DHT manager
    let dht = DhtManager::new(bootstrap);

    // Generate a random target node ID to search for
    let target_id = NodeId::generate();
    println!("🔍 Looking up node: {}", hex::encode(&target_id.0));

    // Find closest known nodes
    let closest = dht.find_node(target_id.clone()).await?;
    println!("📡 Found {} closest nodes:", closest.len());
    for node in &closest {
        println!("   {} @ {}", hex::encode(&node.id.0), node.addr);
    }

    // Store a value in the DHT
    let key = [0xDE; 32];
    let value = b"AAMN DHT value example".to_vec();
    dht.store(key, value.clone()).await?;
    println!("\n💾 Stored value in DHT under key: {}", hex::encode(&key));

    // Retrieve it back
    match dht.get(&key).await? {
        Some(v) => println!("✅ Retrieved: {:?}", std::str::from_utf8(&v)?),
        None => println!("❌ Value not found (expected in standalone example)"),
    }

    println!("\n✨ DHT lookup complete!");
    Ok(())
}
