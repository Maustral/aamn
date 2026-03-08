//! API de Control Remoto de AAMN (gRPC / Protocol Buffers)
//!
//! Excluye un servidor gRPC local que permite interactuar, modificar y ver el
//! estado interno del nodo AAMN desde procesos externos de forma programática.

use crate::metrics::NetworkMetrics;
use crate::network::SecurityEngine;
use crate::routing::RoutingTable;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};

// Incluye el código auto-generado por `tonic-build` desde proto/control.proto
pub mod pb {
    tonic::include_proto!("aamn.control");
}

use pb::node_control_server::{NodeControl, NodeControlServer};
use pb::{ActionResponse, Empty, NodeStatus, Peer, PeerList};

/// Estado interno compartido para el servidor gRPC
pub struct ControlService {
    routing_table: Arc<Mutex<RoutingTable>>,
    engine: Arc<SecurityEngine>,
    metrics: Arc<NetworkMetrics>,
}

impl ControlService {
    pub fn new(
        routing_table: Arc<Mutex<RoutingTable>>,
        engine: Arc<SecurityEngine>,
        metrics: Arc<NetworkMetrics>,
    ) -> Self {
        Self {
            routing_table,
            engine,
            metrics,
        }
    }
}

#[tonic::async_trait]
impl NodeControl for ControlService {
    async fn get_status(&self, _request: Request<Empty>) -> Result<Response<NodeStatus>, Status> {
        let public_key_hex = hex::encode(self.engine.identity.public_id());

        let reply = NodeStatus {
            version: "0.3.0".to_string(),
            public_key_hex,
            active_circuits: 0, // En un futuro leer desde circuit_manager
            connected_peers: self.routing_table.lock().await.get_all_nodes().len() as u32,
            bytes_sent: self.metrics.bytes_encrypted.load(Ordering::Relaxed),
            bytes_received: 0, // Leer de métricas rx
            is_guard: false,   // Por defecto si no esta parseado del config
        };

        Ok(Response::new(reply))
    }

    async fn list_peers(&self, _request: Request<Empty>) -> Result<Response<PeerList>, Status> {
        let nodes = self.routing_table.lock().await.get_all_nodes();
        let mut peers = Vec::new();

        for node in nodes {
            peers.push(Peer {
                node_id_hex: hex::encode(node.id),
                endpoint: node.endpoint,
                latency_ms: node.latency_ms,
                reputation: node.reputation,
            });
        }

        Ok(Response::new(PeerList { peers }))
    }

    async fn generate_noise(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<ActionResponse>, Status> {
        match self.engine.generate_noise_packet() {
            Ok(_packet) => {
                // En un caso real, encolar el paquete para envío en el canal UDP/QUIC
                Ok(Response::new(ActionResponse {
                    success: true,
                    message: "Paquete de ruido forjado exitosamente.".to_string(),
                }))
            }
            Err(e) => Ok(Response::new(ActionResponse {
                success: false,
                message: format!("Error generando paquete: {}", e),
            })),
        }
    }

    async fn stop_node(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<ActionResponse>, Status> {
        // Podría lanzar una señal de parada interna a través de un tokio::sync::Notify
        tracing::warn!("Received StopNode request via gRPC!");
        Ok(Response::new(ActionResponse {
            success: true,
            message: "Parada iniciada".to_string(),
        }))
    }
}

/// Inicia el servidor gRPC y un Gateway REST en segundo plano
pub async fn start_grpc_server(
    port: u16,
    routing_table: Arc<Mutex<RoutingTable>>,
    engine: Arc<SecurityEngine>,
    metrics: Arc<NetworkMetrics>,
) -> anyhow::Result<()> {
    // 1. Iniciar gRPC en port
    let grpc_addr = format!("127.0.0.1:{}", port).parse()?;
    let control_service =
        ControlService::new(routing_table.clone(), engine.clone(), metrics.clone());
    let server = NodeControlServer::new(control_service);

    let _ = tower_http::cors::CorsLayer::permissive();

    tracing::info!("📡 Start gRPC Control API on {}", grpc_addr);
    tokio::spawn(async move {
        let _ = Server::builder()
            .accept_http1(true)
            .add_service(tonic_web::enable(server))
            .serve(grpc_addr)
            .await;
    });

    // 2. Iniciar REST API (REST JSON Gateway) en port + 1 para React Dashboard (Windows friendly)
    use axum::{
        extract::State,
        response::Json,
        routing::{get, post},
        Router,
    };
    use serde_json::{json, Value};

    let rest_port = port + 1;
    let rest_addr = format!("127.0.0.1:{}", rest_port);
    tracing::info!("🌐 Start REST Gateway API on {}", rest_addr);

    #[derive(Clone)]
    struct AppState {
        rt: Arc<Mutex<RoutingTable>>,
        eng: Arc<SecurityEngine>,
        met: Arc<NetworkMetrics>,
    }

    let state = AppState {
        rt: routing_table,
        eng: engine,
        met: metrics,
    };

    let app = Router::new()
        .route("/api/status", get(|State(s): State<AppState>| async move {
            Json(json!({
                "version": "0.3.0",
                "public_key_hex": hex::encode(s.eng.identity.public_id()),
                "active_circuits": 0,
                "connected_peers": s.rt.lock().await.get_all_nodes().len(),
                "bytes_sent": s.met.bytes_encrypted.load(Ordering::Relaxed),
                "bytes_received": 0,
                "is_guard": false
            }))
        }))
        .route("/api/peers", get(|State(s): State<AppState>| async move {
            let nodes = s.rt.lock().await.get_all_nodes();
            let mut peers: Vec<Value> = Vec::new();
            for node in nodes {
                peers.push(json!({
                    "node_id_hex": hex::encode(node.id),
                    "endpoint": node.endpoint,
                    "latency_ms": node.latency_ms,
                    "reputation": node.reputation
                }));
            }
            Json(peers)
        }))
        .route("/api/noise", post(|State(s): State<AppState>| async move {
            match s.eng.generate_noise_packet() {
                Ok(_) => Json(json!({"success": true, "message": "Paquete de ruido forjado exitosamente."})),
                Err(e) => Json(json!({"success": false, "message": format!("Error: {}", e)})),
            }
        }))
        .route("/api/stop", post(|| async {
            tracing::warn!("REST: Received StopNode request!");
            Json(json!({"success": true, "message": "Parada iniciada"}))
        }))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&rest_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
