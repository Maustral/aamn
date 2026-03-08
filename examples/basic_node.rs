//! Example: Start a basic AAMN node
//!
//! This example shows how to initialize a node, connect to bootstrap peers,
//! and keep the node running.
//!
//! Run with:
//!   cargo run --example basic_node

use aamn::{config::Config, daemon::DaemonManager};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Load configuration (or use defaults)
    let config = Config::default();
    tracing::info!("Starting AAMN node on {}", config.network.listen_addr);

    // Create and start the daemon manager
    let daemon = DaemonManager::new();
    let info = daemon
        .start(config.network.listen_addr.port(), false)
        .await?;

    tracing::info!("Node started — PID: {}, Port: {}", info.pid, info.port);

    // Keep running until Ctrl+C
    tracing::info!("Node is running. Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await?;

    // Graceful shutdown
    daemon.stop().await?;
    tracing::info!("Node stopped. Goodbye!");

    Ok(())
}
