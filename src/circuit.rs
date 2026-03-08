use crate::routing::NodeProfile;
use std::time::{Instant, Duration};
use std::sync::Arc;

/// Representa un circuito (ruta de anonimato) activo.
pub struct Circuit {
    pub id: u64,
    pub nodes: Vec<NodeProfile>,
    pub keys: Vec<[u8; 32]>,
    pub created_at: Instant,
    pub expires_at: Instant,
}

impl Circuit {
    pub fn new(nodes: Vec<NodeProfile>, keys: Vec<[u8; 32]>) -> Self {
        let id = rand::random();
        let now = Instant::now();
        Self {
            id,
            nodes,
            keys,
            created_at: now,
            expires_at: now + Duration::from_secs(600), // Circuitos rotan cada 10 min
        }
    }

    pub fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

/// Gestor de Circuitos para rotación constante y gestión de sesiones.
pub struct CircuitManager {
    active_circuits: Vec<Arc<Circuit>>,
}

impl CircuitManager {
    pub fn new() -> Self {
        Self {
            active_circuits: Vec::new(),
        }
    }

    pub fn add_circuit(&mut self, circuit: Circuit) {
        self.active_circuits.push(Arc::new(circuit));
    }

    pub fn get_best_circuit(&self) -> Option<Arc<Circuit>> {
        // Por ahora devuelve el primero no expirado, pero podría ser por carga
        self.active_circuits.iter()
            .find(|c| !c.is_expired())
            .cloned()
    }

    pub fn rotate_circuits(&mut self) {
        self.active_circuits.retain(|c| !c.is_expired());
    }
}
