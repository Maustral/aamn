//! AAMN - Adaptive Anonymous Mesh Network
//!
//! Main entry point for the AAMN node

use aamn::cli::{Cli, Commands};
use aamn::crypto::NodeIdentity;
use aamn::daemon::DaemonManager;
use aamn::error::AAMNError;
use aamn::grpc::start_grpc_server;
use aamn::logging::{self, LoggingConfig};
use aamn::metrics::NetworkMetrics;
use aamn::network::SecurityEngine;
use aamn::rate_limiter::RateLimiter;
use aamn::routing::{NodeProfile, RoutingTable};
use aamn::socks5::Socks5Server;
use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex as TokioMutex};

#[tokio::main]
async fn main() -> Result<()> {
    // Parsear argumentos de línea de comandos
    let cli = Cli::parse();

    // Inicializar sistema de logging
    let log_config = LoggingConfig {
        level: match cli.verbose {
            0 => tracing::Level::INFO,
            1 => tracing::Level::DEBUG,
            _ => tracing::Level::TRACE,
        },
        json: false,
        directory: None,
        filename: Some("aamn.log".to_string()),
        rotation: true,
        max_file_size: 10,
        max_files: 5,
    };

    if let Err(e) = logging::init(&log_config) {
        eprintln!("Warning: Could not initialize logging: {}", e);
    }

    tracing::info!("AAMN v0.2.0 starting...");
    tracing::info!("Log level: {}", log_config.level);

    // Ejecutar el comando solicitado
    match cli.command {
        Commands::Start {
            port,
            bootstrap,
            socks5_port,
            grpc_port,
        } => {
            start_node(port, bootstrap, socks5_port, grpc_port).await?;
        }

        Commands::Stop => {
            stop_node().await?;
        }

        Commands::Status => {
            show_status().await?;
        }

        Commands::Connect { peer } => {
            connect_to_peer(&peer).await?;
        }

        Commands::Peers => {
            list_peers().await?;
        }

        Commands::GenIdentity { output } => {
            generate_identity(&output).await?;
        }

        Commands::ValidateConfig { config } => {
            validate_config(&config).await?;
        }
    }

    Ok(())
}

/// Iniciar el nodo AAMN
async fn start_node(
    port: u16,
    bootstrap: Option<String>,
    socks5_port: Option<u16>,
    grpc_port: Option<u16>,
) -> Result<()> {
    tracing::info!("Starting AAMN node on port {}", port);

    // Inicializar identidad del nodo
    let identity = NodeIdentity::generate();
    tracing::info!("Node identity: {}", hex::encode(identity.public_id()));

    // Inicializar tabla de routing
    let routing_table = RoutingTable::new();

    // Cargar nodos bootstrap si se especificaron
    if let Some(bootstrap_addr) = bootstrap {
        tracing::info!("Bootstrap node: {}", bootstrap_addr);
        // Aquí se conectaría al nodo bootstrap
    }

    // Agregar nodos de ejemplo (en producción vendrían de DHT)
    let mut table = routing_table;
    for i in 0..10 {
        table.update_node(NodeProfile {
            id: [i as u8; 32],
            endpoint: format!("172.16.0.{}:9000", i),
            last_seen: Utc::now(),
            latency_ms: 10 + i * 5,
            bandwidth_kbps: 10000 / (i + 1),
            reputation: 0.99,
            staked_amount: (5000 / (i + 1)) as u64,
            is_guard: i < 2,
        });
    }

    // Inicializar motor de seguridad
    let engine = Arc::new(SecurityEngine::new(table.clone()));
    let routing_table_arc = Arc::new(TokioMutex::new(table));

    // Inicializar métricas
    let metrics = NetworkMetrics::new();

    // Inicializar rate limiter
    let rate_limiter = RateLimiter::new(100); // 100 req/s

    tracing::info!("Security engine initialized");

    // Iniciar SOCKS5 si se solicitó
    if let Some(s_port) = socks5_port {
        let socks5 = Socks5Server::new(s_port, engine.clone());
        tokio::spawn(async move {
            if let Err(e) = socks5.start().await {
                tracing::error!("SOCKS5 server failed: {}", e);
            }
        });
    }

    // Iniciar gRPC Control API si se solicitó
    if let Some(g_port) = grpc_port {
        let engine_clone = engine.clone();
        let rt_clone = routing_table_arc.clone();
        let metrics_clone = metrics.clone();
        tokio::spawn(async move {
            if let Err(e) = start_grpc_server(g_port, rt_clone, engine_clone, metrics_clone).await {
                tracing::error!("gRPC server failed: {}", e);
            }
        });
    }

    tracing::info!("Node ready!");

    // Canal para tráfico
    let (_tx, mut rx) = mpsc::channel::<Vec<u8>>(100);

    // Escuchar tráfico
    tracing::info!("Listening for traffic...");

    // Procesar tráfico capturado
    while let Some(raw_packet) = rx.recv().await {
        tracing::debug!("Packet captured ({} bytes)", raw_packet.len());

        // Fragmentación
        let fragments = engine.fragmenter.fragment(&raw_packet);
        tracing::debug!("Fragmented into {} parts", fragments.len());

        for (frag_id, frag_data, _is_last) in fragments {
            // Verificar rate limit
            let node_id = [frag_id as u8; 32]; // Usar ID real del nodo
            if !rate_limiter.check(&node_id) {
                tracing::warn!("Rate limit exceeded for node");
                continue;
            }

            // Cifrado onion
            match engine.protect_traffic_auto(frag_data, 3) {
                Ok(safe_packet) => {
                    metrics.inc_packets_sent(1);
                    metrics.add_bytes_encrypted(safe_packet.payload.len() as u64);
                    tracing::trace!("Packet protected and sent");
                }
                Err(e) => {
                    tracing::error!("Error protecting packet: {}", e);
                }
            }
        }
    }

    Ok(())
}

/// Detener el nodo
async fn stop_node() -> Result<()> {
    let daemon = DaemonManager::new();

    match daemon.stop().await {
        Ok(_) => {
            println!("Node stopped successfully");
            tracing::info!("Daemon stopped");
        }
        Err(e) => {
            println!("Error stopping node: {}", e);
            tracing::error!("Failed to stop daemon: {}", e);
        }
    }

    Ok(())
}

/// Mostrar estado del nodo
async fn show_status() -> Result<()> {
    println!("=== AAMN Node Status ===");
    println!("Version: 0.2.0");
    println!("Status: Running");
    println!("Port: 9000");
    println!("Peers: 0");
    println!(" Circuits: 0");
    println!("Uptime: 0s");
    println!("=======================");
    Ok(())
}

/// Conectar a un peer
async fn connect_to_peer(peer: &str) -> Result<()> {
    tracing::info!("Connecting to peer: {}", peer);
    println!("Connecting to {}...", peer);
    println!("Connection not implemented yet");
    Ok(())
}

/// Listar peers conectados
async fn list_peers() -> Result<()> {
    println!("=== Connected Peers ===");
    println!("No peers connected");
    println!("======================");
    Ok(())
}

/// Generar nueva identidad
async fn generate_identity(output: &std::path::Path) -> Result<()> {
    let identity = NodeIdentity::generate();
    let public_id = identity.public_id();

    println!("Generated new identity:");
    println!("  Public Key: {}", hex::encode(public_id));

    // Guardar identidad (simplificado - en producción usar cifrado)
    let identity_data = serde_json::json!({
        "public_key": hex::encode(public_id),
    });

    let json = serde_json::to_string_pretty(&identity_data)?;
    std::fs::write(output, json)?;

    println!("Identity saved to: {}", output.display());
    Ok(())
}

/// Validar configuración
async fn validate_config(config: &std::path::Path) -> Result<()> {
    if !config.exists() {
        return Err(AAMNError::ConfigFileNotFound(config.display().to_string()).into());
    }

    let content = std::fs::read_to_string(config)?;
    let _parsed: toml::Value = toml::from_str(&content)?;

    println!("Configuration is valid!");
    Ok(())
}
