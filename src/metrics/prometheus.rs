//! AAMN Prometheus Metrics - Exposición de métricas para Prometheus
//!
//! Este módulo proporciona un endpoint HTTP para exponer métricas
//! en formato Prometheus.

use crate::metrics::NetworkMetrics;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Métricas formateadas para Prometheus
pub struct PrometheusExporter {
    metrics: Arc<NetworkMetrics>,
    traffic_collector: Arc<RwLock<HashMap<[u8; 32], NodeTraffic>>>,
}

#[derive(Debug, Clone)]
pub struct NodeTraffic {
    pub packets_sent: u64,
    pub packets_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub latency_avg_ms: f64,
    pub last_update: std::time::Instant,
}

impl PrometheusExporter {
    pub fn new(metrics: Arc<NetworkMetrics>) -> Self {
        Self {
            metrics,
            traffic_collector: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Genera el formato Prometheus de todas las métricas
    pub async fn generate_metrics(&self) -> String {
        let mut output = String::new();

        // Métricas globales de red
        output.push_str("# HELP aamn_packets_sent_total Total packets sent\n");
        output.push_str("# TYPE aamn_packets_sent_total counter\n");
        output.push_str(&format!(
            "aamn_packets_sent_total {}\n",
            self.metrics.packets_sent.load(std::sync::atomic::Ordering::Relaxed)
        ));

        output.push_str("# HELP aamn_packets_received_total Total packets received\n");
        output.push_str("# TYPE aamn_packets_received_total counter\n");
        output.push_str(&format!(
            "aamn_packets_received_total {}\n",
            self.metrics.packets_received.load(std::sync::atomic::Ordering::Relaxed)
        ));

        output.push_str("# HELP aamn_bytes_encrypted_total Total bytes encrypted\n");
        output.push_str("# TYPE aamn_bytes_encrypted_total counter\n");
        output.push_str(&format!(
            "aamn_bytes_encrypted_total {}\n",
            self.metrics.bytes_encrypted.load(std::sync::atomic::Ordering::Relaxed)
        ));

        output.push_str("# HELP aamn_bytes_decrypted_total Total bytes decrypted\n");
        output.push_str("# TYPE aamn_bytes_decrypted_total counter\n");
        output.push_str(&format!(
            "aamn_bytes_decrypted_total {}\n",
            self.metrics.bytes_decrypted.load(std::sync::atomic::Ordering::Relaxed)
        ));

        output.push_str("# HELP aamn_active_circuits Current number of active circuits\n");
        output.push_str("# TYPE aamn_active_circuits gauge\n");
        output.push_str(&format!(
            "aamn_active_circuits {}\n",
            self.metrics.active_circuits.load(std::sync::atomic::Ordering::Relaxed)
        ));

        output.push_str("# HELP aamn_fragments_processed_total Total fragments processed\n");
        output.push_str("# TYPE aamn_fragments_processed_total counter\n");
        output.push_str(&format!(
            "aamn_fragments_processed_total {}\n",
            self.metrics.fragments_processed.load(std::sync::atomic::Ordering::Relaxed)
        ));

        output.push_str("# HELP aamn_validation_errors_total Total validation errors\n");
        output.push_str("# TYPE aamn_validation_errors_total counter\n");
        output.push_str(&format!(
            "aamn_validation_errors_total {}\n",
            self.metrics.validation_errors.load(std::sync::atomic::Ordering::Relaxed)
        ));

        output.push_str("# HELP aamn_connection_failures_total Total connection failures\n");
        output.push_str("# TYPE aamn_connection_failures_total counter\n");
        output.push_str(&format!(
            "aamn_connection_failures_total {}\n",
            self.metrics.connection_failures.load(std::sync::atomic::Ordering::Relaxed)
        ));

        // Métricas por nodo
        let traffic = self.traffic_collector.read().await;
        for (node_id, node_traffic) in traffic.iter() {
            let node_id_hex = hex::encode(node_id);
            
            output.push_str(&format!(
                "# HELP aamn_node_packets_sent{{node=\"{}\"}} Node packets sent\n",
                node_id_hex
            ));
            output.push_str(&format!(
                "aamn_node_packets_sent{{node=\"{}\"}} {}\n",
                node_id_hex, node_traffic.packets_sent
            ));
            
            output.push_str(&format!(
                "# HELP aamn_node_bytes_sent{{node=\"{}\"}} Node bytes sent\n",
                node_id_hex
            ));
            output.push_str(&format!(
                "aamn_node_bytes_sent{{node=\"{}\"}} {}\n",
                node_id_hex, node_traffic.bytes_sent
            ));
            
            output.push_str(&format!(
                "# HELP aamn_node_latency_avg{{node=\"{}\"}} Node average latency in ms\n",
                node_id_hex
            ));
            output.push_str(&format!(
                "aamn_node_latency_avg{{node=\"{}\"}} {}\n",
                node_id_hex, node_traffic.latency_avg_ms
            ));
        }

        output
    }
}

/// Health check endpoint
pub struct HealthChecker {
    started_at: std::time::Instant,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            started_at: std::time::Instant::now(),
        }
    }

    /// Retorna el estado de salud del nodo
    pub fn health_check(&self) -> HealthStatus {
        let uptime = self.started_at.elapsed();
        
        HealthStatus {
            status: HealthState::Healthy,
            uptime_secs: uptime.as_secs(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthStatus {
    pub status: HealthState,
    pub uptime_secs: u64,
    pub version: String,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub enum HealthState {
    Healthy,
    Degraded,
    Unhealthy,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_prometheus_exporter() {
        let metrics = NetworkMetrics::new();
        let exporter = PrometheusExporter::new(metrics);
        
        let output = exporter.generate_metrics().await;
        
        assert!(output.contains("aamn_packets_sent_total"));
        assert!(output.contains("aamn_active_circuits"));
    }

    #[test]
    fn test_health_checker() {
        let checker = HealthChecker::new();
        let status = checker.health_check();
        
        assert_eq!(status.status, HealthState::Healthy);
        assert!(status.uptime_secs >= 0);
    }
}
