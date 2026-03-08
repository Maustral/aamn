use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Identidad y métricas de un nodo para el routing adaptativo.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeProfile {
    pub id: [u8; 32],
    pub endpoint: String, // IP:Port
    pub last_seen: DateTime<Utc>,
    pub latency_ms: u32,
    pub bandwidth_kbps: u32,
    pub reputation: f32,    // 0.0 a 1.0
    pub staked_amount: u64, // Cantidad de tokens depositados (Proof-of-Stake)
    #[serde(default)]
    pub is_guard: bool, // Si este nodo está autorizado como guard node (entry)
    /// Si este nodo puede actuar como nodo de entrada.
    #[serde(default = "default_can_enter")]
    pub can_enter: bool,
    /// Si este nodo puede actuar como nodo intermedio.
    #[serde(default = "default_can_middle")]
    pub can_middle: bool,
    /// Si este nodo puede actuar como nodo de salida.
    #[serde(default = "default_can_exit")]
    pub can_exit: bool,
}

fn default_can_enter() -> bool {
    true
}

fn default_can_middle() -> bool {
    true
}

fn default_can_exit() -> bool {
    true
}

/// Tabla de Routing basada en DHT (Simplificada para el prototipo).
/// En una implementación real, esto usaría buckets de Kademlia.
#[derive(Clone)]
pub struct RoutingTable {
    nodes: HashMap<[u8; 32], NodeProfile>,
    guard_nodes: Vec<[u8; 32]>, // IDs de nodos guardianes
}

impl Default for RoutingTable {
    fn default() -> Self {
        Self::new()
    }
}

impl RoutingTable {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            guard_nodes: Vec::new(),
        }
    }

    pub fn update_node(&mut self, profile: NodeProfile) {
        if profile.is_guard && !self.guard_nodes.contains(&profile.id) {
            self.guard_nodes.push(profile.id);
        }
        self.nodes.insert(profile.id, profile);
    }

    pub fn get_all_nodes(&self) -> Vec<NodeProfile> {
        self.nodes.values().cloned().collect()
    }

    pub fn get_guards(&self) -> Vec<NodeProfile> {
        self.guard_nodes
            .iter()
            .filter_map(|id| self.nodes.get(id))
            .cloned()
            .collect()
    }

    /// Guarda la tabla de routing en disco para persistencia entre reinicios.
    pub fn save_to_disk(&self, path: &str) -> Result<()> {
        let data = serde_json::to_string_pretty(&self.nodes)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    /// ✅ CORRECCIÓN 2.1: Carga la tabla de routing desde disco correctamente
    pub fn load_from_disk(path: &str) -> Result<Self> {
        let data = std::fs::read_to_string(path)?;
        let nodes: HashMap<[u8; 32], NodeProfile> = serde_json::from_str(&data)?;
        let guard_nodes = nodes
            .values()
            .filter(|n| n.is_guard)
            .map(|n| n.id)
            .collect();
        Ok(Self { nodes, guard_nodes })
    }
}

/// Motor de Selección de Rutas Probabilístico.
pub struct PathFinder {
    routing_table: RoutingTable,
}

impl PathFinder {
    pub fn new(routing_table: RoutingTable) -> Self {
        Self { routing_table }
    }

    /// Selecciona una ruta de N saltos basada en entropía y rendimiento del nodo.
    /// Implementa "Entropy-Weighted Path Selection".
    pub fn find_probabilistic_path(&self, hops: usize) -> Result<Vec<NodeProfile>> {
        let all_nodes = self.routing_table.get_all_nodes();

        if all_nodes.len() < hops {
            return Err(anyhow!(
                "No hay suficientes nodos en la DHT para crear la ruta"
            ));
        }

        let mut path = Vec::new();
        let mut available_nodes = all_nodes;
        let mut rng = rand::thread_rng();

        if hops == 0 {
            return Ok(path);
        }

        // --- Primer salto: preferir guards que puedan actuar como entrada ---
        let mut start_idx = 0;
        let mut entry_candidates: Vec<NodeProfile> = self
            .routing_table
            .get_guards()
            .into_iter()
            .filter(|n| n.can_enter)
            .collect();

        if entry_candidates.is_empty() {
            entry_candidates = available_nodes
                .iter()
                .filter(|n| n.can_enter)
                .cloned()
                .collect();
        }

        if !entry_candidates.is_empty() {
            let idx = rng.gen_range(0..entry_candidates.len());
            let guard = entry_candidates[idx].clone();
            if let Some(pos) = available_nodes.iter().position(|n| n.id == guard.id) {
                available_nodes.remove(pos);
            }
            path.push(guard);
            start_idx = 1;
        }

        // --- Hops intermedios y de salida ---
        for i in start_idx..hops {
            // Último hop: debe poder ser salida
            let is_last = i == hops - 1;

            let role_filtered: Vec<NodeProfile> = available_nodes
                .iter()
                .filter(|n| if is_last { n.can_exit } else { n.can_middle })
                .cloned()
                .collect();

            let candidates = if !role_filtered.is_empty() {
                role_filtered
            } else {
                // Fallback: si no hay nodos con el rol deseado, usar todos los disponibles
                available_nodes.clone()
            };

            // Calculamos los pesos para la selección probabilística
            let weights: Vec<f32> = candidates
                .iter()
                .map(|n| {
                    let latency_score = 1000.0 / (n.latency_ms as f32).max(1.0);
                    let bw_score = n.bandwidth_kbps as f32 / 1000.0;
                    let stake_score = (n.staked_amount as f32 / 1000.0).min(1.0); // Normalizado

                    (n.reputation * 0.4)
                        + (stake_score * 0.3)
                        + (latency_score * 0.2)
                        + (bw_score * 0.1)
                })
                .collect();

            if candidates.is_empty() {
                return Err(anyhow!("No hay nodos suficientes para construir la ruta"));
            }

            // Selección ponderada (Weighted Random Choice)
            let selected = if let Ok(selected_index) = self.weighted_choice(&weights, &mut rng) {
                candidates[selected_index].clone()
            } else {
                // Si falla la selección ponderada, elegimos uno al azar para mantener la entropía
                let index = rng.gen_range(0..candidates.len());
                candidates[index].clone()
            };

            // Eliminar del conjunto disponible y añadir al path
            if let Some(pos) = available_nodes.iter().position(|n| n.id == selected.id) {
                available_nodes.remove(pos);
            }
            path.push(selected);
        }

        Ok(path)
    }

    fn weighted_choice(&self, weights: &[f32], rng: &mut impl Rng) -> Result<usize> {
        let sum: f32 = weights.iter().sum();
        if sum <= 0.0 {
            return Err(anyhow!("Pesos inválidos"));
        }

        let mut chosen = rng.gen_range(0.0..sum);
        for (i, &w) in weights.iter().enumerate() {
            chosen -= w;
            if chosen <= 0.0 {
                return Ok(i);
            }
        }
        Ok(weights.len() - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probabilistic_routing() -> Result<()> {
        let mut table = RoutingTable::new();

        // Poblamos con nodos de prueba
        for i in 0..10 {
            table.update_node(NodeProfile {
                id: [i as u8; 32],
                endpoint: format!("192.168.1.{}:9000", i),
                last_seen: Utc::now(),
                latency_ms: 10 + (i * 5) as u32,
                bandwidth_kbps: 1000 - (i * 50) as u32,
                reputation: 0.9 - (i as f32 * 0.05),
                staked_amount: 1000 - (i * 100) as u64,
                is_guard: i == 0, // El primer nodo es el único guard
                can_enter: true,
                can_middle: true,
                can_exit: true,
            });
        }

        let finder = PathFinder::new(table);

        // Generamos 100 rutas para verificar la distribución probabilística
        let mut first_hop_counts = HashMap::new();
        for _ in 0..100 {
            let path = finder.find_probabilistic_path(3)?;
            assert_eq!(path.len(), 3);
            *first_hop_counts.entry(path[0].id).or_insert(0) += 1;
        }

        // El nodo 0 tiene mejores métricas, debería aparecer más veces que el nodo 9
        let count_best = *first_hop_counts.get(&[0u8; 32]).unwrap_or(&0);
        let count_worst = *first_hop_counts.get(&[9u8; 32]).unwrap_or(&0);

        println!(
            "Apariciones del mejor nodo: {}, Peor nodo: {}",
            count_best, count_worst
        );
        assert!(
            count_best >= count_worst,
            "El ruteo probabilístico falló en favorecer nodos de alto rendimiento"
        );

        Ok(())
    }
}
