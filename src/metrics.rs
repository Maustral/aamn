/// ✅ IMPLEMENTACIÓN 3.2: Sistema de Métricas y Monitoreo
/// 
/// Sistema para recolectar y analizar métricas de red, incluyendo
/// paquetes procesados, bytes cifrados, latencia, y estado de circuitos.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::SystemTime;

/// ✅ IMPLEMENTACIÓN 3.2: Métricas de la red AAMN
#[derive(Clone, Debug)]
pub struct NetworkMetrics {
    /// Total de paquetes enviados
    pub packets_sent: Arc<AtomicU64>,
    /// Total de paquetes recibidos
    pub packets_received: Arc<AtomicU64>,
    /// Total de bytes cifrados
    pub bytes_encrypted: Arc<AtomicU64>,
    /// Total de bytes descifrados
    pub bytes_decrypted: Arc<AtomicU64>,
    /// Número actual de circuitos activos
    pub active_circuits: Arc<AtomicU64>,
    /// Número de fragmentos procesados
    pub fragments_processed: Arc<AtomicU64>,
    /// Número de errores de validación (HMAC inválido, etc.)
    pub validation_errors: Arc<AtomicU64>,
    /// Número de intentos de conexión fallidos
    pub connection_failures: Arc<AtomicU64>,
    /// Timestamp de creación de estas métricas
    pub created_at: SystemTime,
}

impl NetworkMetrics {
    /// Crea una nueva instancia de métricas
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            packets_sent: Arc::new(AtomicU64::new(0)),
            packets_received: Arc::new(AtomicU64::new(0)),
            bytes_encrypted: Arc::new(AtomicU64::new(0)),
            bytes_decrypted: Arc::new(AtomicU64::new(0)),
            active_circuits: Arc::new(AtomicU64::new(0)),
            fragments_processed: Arc::new(AtomicU64::new(0)),
            validation_errors: Arc::new(AtomicU64::new(0)),
            connection_failures: Arc::new(AtomicU64::new(0)),
            created_at: SystemTime::now(),
        })
    }

    /// Incrementa el contador de paquetes enviados
    pub fn inc_packets_sent(&self, count: u64) {
        self.packets_sent.fetch_add(count, Ordering::Relaxed);
    }

    /// Incrementa el contador de paquetes recibidos
    pub fn inc_packets_received(&self, count: u64) {
        self.packets_received.fetch_add(count, Ordering::Relaxed);
    }

    /// Registra bytes cifrados
    pub fn add_bytes_encrypted(&self, bytes: u64) {
        self.bytes_encrypted.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Registra bytes descifrados
    pub fn add_bytes_decrypted(&self, bytes: u64) {
        self.bytes_decrypted.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Incrementa el contador de circuitos activos
    pub fn inc_active_circuits(&self) {
        self.active_circuits.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrementa el contador de circuitos activos
    pub fn dec_active_circuits(&self) {
        self.active_circuits.fetch_sub(1, Ordering::Relaxed);
    }

    /// Incrementa el contador de fragmentos procesados
    pub fn inc_fragments_processed(&self, count: u64) {
        self.fragments_processed.fetch_add(count, Ordering::Relaxed);
    }

    /// Registra un error de validación
    pub fn inc_validation_errors(&self) {
        self.validation_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Registra un fallo de conexión
    pub fn inc_connection_failures(&self) {
        self.connection_failures.fetch_add(1, Ordering::Relaxed);
    }

    /// Obtiene un resumen de las métricas actuales
    pub fn summary(&self) -> MetricsSummary {
        let uptime = self.created_at.elapsed().unwrap_or_default();
        let packets_sent = self.packets_sent.load(Ordering::Relaxed);
        let packets_received = self.packets_received.load(Ordering::Relaxed);
        let bytes_encrypted = self.bytes_encrypted.load(Ordering::Relaxed);
        let bytes_decrypted = self.bytes_decrypted.load(Ordering::Relaxed);
        
        MetricsSummary {
            uptime_secs: uptime.as_secs(),
            packets_sent,
            packets_received,
            bytes_encrypted,
            bytes_decrypted,
            active_circuits: self.active_circuits.load(Ordering::Relaxed),
            fragments_processed: self.fragments_processed.load(Ordering::Relaxed),
            validation_errors: self.validation_errors.load(Ordering::Relaxed),
            connection_failures: self.connection_failures.load(Ordering::Relaxed),
            throughput_packets_per_sec: if uptime.as_secs() > 0 {
                packets_sent / uptime.as_secs()
            } else {
                0
            },
            throughput_bytes_per_sec: if uptime.as_secs() > 0 {
                bytes_encrypted / uptime.as_secs()
            } else {
                0
            },
        }
    }

    /// Reinicia todas las métricas
    pub fn reset(&self) {
        self.packets_sent.store(0, Ordering::Relaxed);
        self.packets_received.store(0, Ordering::Relaxed);
        self.bytes_encrypted.store(0, Ordering::Relaxed);
        self.bytes_decrypted.store(0, Ordering::Relaxed);
        self.active_circuits.store(0, Ordering::Relaxed);
        self.fragments_processed.store(0, Ordering::Relaxed);
        self.validation_errors.store(0, Ordering::Relaxed);
        self.connection_failures.store(0, Ordering::Relaxed);
    }
}

impl Default for NetworkMetrics {
    fn default() -> Self {
        Self::new().as_ref().clone()
    }
}

/// ✅ IMPLEMENTACIÓN 3.2: Resumen de métricas para reporting
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    pub uptime_secs: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub bytes_encrypted: u64,
    pub bytes_decrypted: u64,
    pub active_circuits: u64,
    pub fragments_processed: u64,
    pub validation_errors: u64,
    pub connection_failures: u64,
    pub throughput_packets_per_sec: u64,
    pub throughput_bytes_per_sec: u64,
}

impl std::fmt::Display for MetricsSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "=== AAMN Network Metrics ===\n\
             Uptime: {}s\n\
             Packets sent: {}\n\
             Packets received: {}\n\
             Bytes encrypted: {} (throughput: {}/s)\n\
             Bytes decrypted: {}\n\
             Active circuits: {}\n\
             Fragments processed: {}\n\
             Validation errors: {}\n\
             Connection failures: {}\n\
             Avg throughput: {:.2} packets/s",
            self.uptime_secs,
            self.packets_sent,
            self.packets_received,
            self.bytes_encrypted,
            self.throughput_bytes_per_sec,
            self.bytes_decrypted,
            self.active_circuits,
            self.fragments_processed,
            self.validation_errors,
            self.connection_failures,
            self.throughput_packets_per_sec as f64,
        )
    }
}

/// ✅ IMPLEMENTACIÓN 3.2: Métricas de tráfico por nodo
#[derive(Clone, Debug)]
pub struct NodeTrafficMetrics {
    /// ID del nodo
    pub node_id: [u8; 32],
    /// Paquetes enviados a este nodo
    pub packets_sent: u64,
    /// Paquetes recibidos de este nodo
    pub packets_received: u64,
    /// Bytes enviados a este nodo
    pub bytes_sent: u64,
    /// Bytes recibidos de este nodo
    pub bytes_received: u64,
    /// Latencia promedio en milisegundos
    pub avg_latency_ms: u32,
}

/// ✅ IMPLEMENTACIÓN 3.2: Recolector de métricas por nodo
pub struct TrafficMetricsCollector {
    /// HashMap: Node ID -> métricas de tráfico
    metrics: std::sync::Mutex<std::collections::HashMap<[u8; 32], NodeTrafficMetrics>>,
}

impl TrafficMetricsCollector {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            metrics: std::sync::Mutex::new(std::collections::HashMap::new()),
        })
    }

    /// Registra un paquete enviado a un nodo
    pub fn record_send(&self, node_id: &[u8; 32], bytes: u64, latency_ms: u32) {
        let mut metrics = self.metrics.lock().unwrap();
        let entry = metrics.entry(*node_id)
            .or_insert(NodeTrafficMetrics {
                node_id: *node_id,
                packets_sent: 0,
                packets_received: 0,
                bytes_sent: 0,
                bytes_received: 0,
                avg_latency_ms: 0,
            });

        entry.packets_sent += 1;
        entry.bytes_sent += bytes;
        // Actualizar promedio exponencial de latencia
        let new_avg = ((entry.avg_latency_ms as u64) * 3 + (latency_ms as u64)) / 4;
        entry.avg_latency_ms = new_avg as u32;
    }

    /// Registra un paquete recibido de un nodo
    pub fn record_receive(&self, node_id: &[u8; 32], bytes: u64) {
        let mut metrics = self.metrics.lock().unwrap();
        let entry = metrics.entry(*node_id)
            .or_insert(NodeTrafficMetrics {
                node_id: *node_id,
                packets_sent: 0,
                packets_received: 0,
                bytes_sent: 0,
                bytes_received: 0,
                avg_latency_ms: 0,
            });

        entry.packets_received += 1;
        entry.bytes_received += bytes;
    }

    /// Obtiene todas las métricas registradas
    pub fn get_all(&self) -> Vec<NodeTrafficMetrics> {
        self.metrics.lock().unwrap().values().cloned().collect()
    }

    /// Obtiene métricas de un nodo específico
    pub fn get_node(&self, node_id: &[u8; 32]) -> Option<NodeTrafficMetrics> {
        self.metrics.lock().unwrap().get(node_id).cloned()
    }
}

// Nota: TrafficMetricsCollector no implementa Default
// Usar TrafficMetricsCollector::new() en su lugar

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_network_metrics() {
        let metrics = NetworkMetrics::new();
        
        metrics.inc_packets_sent(5);
        metrics.inc_packets_received(3);
        metrics.add_bytes_encrypted(1024);
        metrics.inc_active_circuits();
        
        assert_eq!(metrics.packets_sent.load(Ordering::Relaxed), 5);
        assert_eq!(metrics.packets_received.load(Ordering::Relaxed), 3);
        assert_eq!(metrics.bytes_encrypted.load(Ordering::Relaxed), 1024);
        assert_eq!(metrics.active_circuits.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_metrics_summary() {
        let metrics = NetworkMetrics::new();
        metrics.inc_packets_sent(100);
        metrics.add_bytes_encrypted(10240);
        
        thread::sleep(Duration::from_millis(100));
        
        let summary = metrics.summary();
        assert_eq!(summary.packets_sent, 100);
        assert_eq!(summary.bytes_encrypted, 10240);
    }
}
