use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use rand::Rng;
use anyhow::{Result, anyhow};

/// Identidad y métricas de un nodo para el routing adaptativo.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeProfile {
    pub id: [u8; 32],
    pub endpoint: String, // IP:Port
    pub last_seen: DateTime<Utc>,
    pub latency_ms: u32,
    pub bandwidth_kbps: u32,
    pub reputation: f32, // 0.0 a 1.0
    pub staked_amount: u64, // Cantidad de tokens depositados (Proof-of-Stake)
}

/// Tabla de Routing basada en DHT (Simplificada para el prototipo).
/// En una implementación real, esto usaría buckets de Kademlia.
pub struct RoutingTable {
    nodes: HashMap<[u8; 32], NodeProfile>,
}

impl RoutingTable {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn update_node(&mut self, profile: NodeProfile) {
        self.nodes.insert(profile.id, profile);
    }

    pub fn get_all_nodes(&self) -> Vec<NodeProfile> {
        self.nodes.values().cloned().collect()
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
        Ok(Self { nodes })
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
            return Err(anyhow!("No hay suficientes nodos en la DHT para crear la ruta"));
        }

        let mut path = Vec::new();
        let mut available_nodes = all_nodes;
        let mut rng = rand::thread_rng();

        for _ in 0..hops {
            // Calculamos los pesos para la selección probabilística
            // Score = (Reputación * 0.4) + (Stake * 0.3) + ((1 / Latencia) * 0.2) + (AnchoDeBanda * 0.1)
            let weights: Vec<f32> = available_nodes.iter().map(|n| {
                let latency_score = 1000.0 / (n.latency_ms as f32).max(1.0);
                let bw_score = n.bandwidth_kbps as f32 / 1000.0;
                let stake_score = (n.staked_amount as f32 / 1000.0).min(1.0); // Normalizado
                
                (n.reputation * 0.4) + (stake_score * 0.3) + (latency_score * 0.2) + (bw_score * 0.1)
            }).collect();

            // Selección ponderada (Weighted Random Choice)
            if let Ok(selected_index) = self.weighted_choice(&weights, &mut rng) {
                let node = available_nodes.remove(selected_index);
                path.push(node);
            } else {
                // Si falla la selección ponderada, elegimos uno al azar para mantener la entropía
                let index = rng.gen_range(0..available_nodes.len());
                let node = available_nodes.remove(index);
                path.push(node);
            }
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
        
        println!("Apariciones del mejor nodo: {}, Peor nodo: {}", count_best, count_worst);
        assert!(count_best >= count_worst, "El ruteo probabilístico falló en favorecer nodos de alto rendimiento");

        Ok(())
    }
}
