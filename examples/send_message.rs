//! Example: Send an anonymous message through a 3-hop onion circuit
//!
//! This example demonstrates:
//! - Generating a node identity
//! - Performing a Noise IKpsk2 handshake
//! - Wrapping a payload in 3 onion encryption layers
//! - Inspecting the fixed-size cell output
//!
//! Run with:
//!   cargo run --example send_message

use aamn::{
    crypto::{NodeIdentity, OnionEncryptor},
    handshake::HandshakeManager,
    padding::{Cell, CellType},
};

fn main() -> anyhow::Result<()> {
    // ── Step 1: Generate node identity ──────────────────────────────
    let identity = NodeIdentity::generate();
    println!("✅ Node ID: {}", hex::encode(identity.node_id()));
    println!("   Public key: {}", hex::encode(identity.public_key()));

    // ── Step 2: Prepare session keys (normally from handshake) ──────
    // In production these come from the Noise IKpsk2 handshake.
    // For this example we use fixed demo keys.
    let relay_keys: &[[u8; 32]] = &[
        [0xAA; 32], // key for hop 1
        [0xBB; 32], // key for hop 2
        [0xCC; 32], // key for hop 3
    ];
    let relay_ids: &[[u8; 32]] = &[
        [0x01; 32], // relay 1 node ID
        [0x02; 32], // relay 2 node ID
        [0x03; 32], // relay 3 node ID
    ];

    // ── Step 3: Wrap message in 3 onion layers ──────────────────────
    let message = b"Hello, anonymous world! This is a secret message.";
    println!("\n📨 Original message ({} bytes):", message.len());
    println!("   {:?}", std::str::from_utf8(message)?);

    let encryptor = OnionEncryptor::new(relay_keys);
    let wrapped = encryptor.wrap(message, relay_ids)?;

    println!("\n🧅 After 3-layer onion encryption:");
    println!("   Wrapped size: {} bytes", wrapped.len());
    println!("   First 32 bytes (hex): {}", hex::encode(&wrapped[..32]));

    // ── Step 4: Pack into a fixed-size 512-byte cell ────────────────
    let cell = Cell::new(42, CellType::Data, 0, &wrapped)?;
    let cell_bytes = cell.to_bytes();

    println!("\n📦 Cell (fixed 512 bytes):");
    println!("   Size: {} bytes ✅", cell_bytes.len());
    println!("   Circuit ID: 42");
    println!("   Cell type: Data");

    println!("\n✨ Message wrapped and ready to transmit anonymously!");
    Ok(())
}
