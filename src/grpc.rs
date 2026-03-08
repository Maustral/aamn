//! API de Control Remoto de AAMN (gRPC / Protocol Buffers)
//!
//! Excluye un servidor gRPC local que permite interactuar, modificar y ver el
//! estado interno del nodo AAMN desde procesos externos de forma programática.

use crate::metrics::NetworkMetrics;
use crate::network::SecurityEngine;
use crate::routing::RoutingTable;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::env;
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
    admin_token: Option<String>,
}

impl ControlService {
    pub fn new(
        routing_table: Arc<Mutex<RoutingTable>>,
        engine: Arc<SecurityEngine>,
        metrics: Arc<NetworkMetrics>,
        admin_token: Option<String>,
    ) -> Self {
        Self {
            routing_table,
            engine,
            metrics,
            admin_token,
        }
    }

    /// Autorización simple basada en token para la API de control gRPC.
    /// Si no hay token configurado, se permite el acceso (modo legacy),
    /// but in production se recomienda siempre establecer AAMN_CONTROL_TOKEN.
    #[allow(clippy::result_large_err)]
    fn authorize<T>(&self, req: &Request<T>) -> Result<(), Status> {
        if let Some(expected) = &self.admin_token {
            let metadata = req.metadata();
            if let Some(value) = metadata.get("authorization") {
                if let Ok(header) = value.to_str() {
                    let expected_header = format!("Bearer {}", expected);
                    if header == expected_header {
                        return Ok(());
                    }
                }
            }
            return Err(Status::unauthenticated(
                "missing or invalid admin token for control API",
            ));
        }
        Ok(())
    }
}

#[tonic::async_trait]
impl NodeControl for ControlService {
    async fn get_status(&self, request: Request<Empty>) -> Result<Response<NodeStatus>, Status> {
        self.authorize(&request)?;

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

    async fn list_peers(&self, request: Request<Empty>) -> Result<Response<PeerList>, Status> {
        self.authorize(&request)?;

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
        request: Request<Empty>,
    ) -> Result<Response<ActionResponse>, Status> {
        self.authorize(&request)?;

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

    async fn stop_node(&self, request: Request<Empty>) -> Result<Response<ActionResponse>, Status> {
        self.authorize(&request)?;

        // Podría lanzar una señal de parada interna a través de un tokio::sync::Notify
        tracing::warn!("Received StopNode request via gRPC!");
        Ok(Response::new(ActionResponse {
            success: true,
            message: "Parada iniciada".to_string(),
        }))
    }
}

#[derive(Clone)]
struct AppState {
    rt: Arc<Mutex<RoutingTable>>,
    eng: Arc<SecurityEngine>,
    met: Arc<NetworkMetrics>,
    admin_token: Option<String>,
}

/// Inicia el servidor gRPC y un Gateway REST en segundo plano
pub async fn start_grpc_server(
    port: u16,
    routing_table: Arc<Mutex<RoutingTable>>,
    engine: Arc<SecurityEngine>,
    metrics: Arc<NetworkMetrics>,
) -> anyhow::Result<()> {
    // Token de control opcional para proteger las APIs de administración.
    // Si está presente, todas las llamadas gRPC/REST deberán incluir:
    //   Authorization: Bearer <AAMN_CONTROL_TOKEN>
    let control_token = env::var("AAMN_CONTROL_TOKEN").ok();
    if control_token.is_none() {
        tracing::warn!(
            "AAMN_CONTROL_TOKEN is not set - Control APIs (gRPC/REST) are running WITHOUT authentication. \
             Do NOT expose ports 50051/50052 publicly in production."
        );
    }

    // 1. Iniciar gRPC en port
    let grpc_addr = format!("127.0.0.1:{}", port).parse()?;
    let control_service = ControlService::new(
        routing_table.clone(),
        engine.clone(),
        metrics.clone(),
        control_token.clone(),
    );
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
    let rest_port = port + 1;
    let rest_addr = format!("127.0.0.1:{}", rest_port);
    tracing::info!("🌐 Start REST Gateway API on {}", rest_addr);

    let state = AppState {
        rt: routing_table,
        eng: engine,
        met: metrics,
        admin_token: control_token,
    };

    let app = Router::new()
        .route("/api/status", get(rest_status))
        .route("/api/peers", get(rest_peers))
        .route("/api/noise", post(rest_noise))
        .route("/api/stop", post(rest_stop))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&rest_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Autorización basada en token para la REST API.
fn authorize_rest(headers: &HeaderMap, state: &AppState) -> Result<(), StatusCode> {
    if let Some(expected) = &state.admin_token {
        if let Some(value) = headers.get("authorization") {
            if let Ok(header) = value.to_str() {
                let expected_header = format!("Bearer {}", expected);
                if header == expected_header {
                    return Ok(());
                }
            }
        }
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(())
}

async fn rest_status(
    State(s): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, StatusCode> {
    authorize_rest(&headers, &s)?;

    Ok(Json(json!({
        "version": "0.3.0",
        "public_key_hex": hex::encode(s.eng.identity.public_id()),
        "active_circuits": 0,
        "connected_peers": s.rt.lock().await.get_all_nodes().len(),
        "bytes_sent": s.met.bytes_encrypted.load(Ordering::Relaxed),
        "bytes_received": 0,
        "is_guard": false
    })))
}

async fn rest_peers(
    State(s): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, StatusCode> {
    authorize_rest(&headers, &s)?;

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
    Ok(Json(Value::Array(peers)))
}

async fn rest_noise(
    State(s): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, StatusCode> {
    authorize_rest(&headers, &s)?;

    match s.eng.generate_noise_packet() {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "message": "Paquete de ruido forjado exitosamente."
        }))),
        Err(e) => Ok(Json(json!({
            "success": false,
            "message": format!("Error: {}", e)
        }))),
    }
}

async fn rest_stop(
    State(s): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, StatusCode> {
    // Esta operación es crítica; protegemos con el mismo token
    // y además se recomienda protección adicional a nivel de red
    // (firewall, VPN, etc.).
    authorize_rest(&headers, &s)?;
    tracing::warn!("REST: Received StopNode request!");
    Ok(Json(json!({
        "success": true,
        "message": "Parada iniciada"
    })))
}
