use crate::circuit::CircuitManager;
use crate::crypto::{NodeIdentity, OnionEncryptor};
use crate::fragment::FragmentationManager;
use crate::protocol::AAMNPacket;
use crate::rate_limiter::RateLimiter;
use crate::routing::{PathFinder, RoutingTable};
use anyhow::Result;
use rand::Rng;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;

/// ✅ FASE 2.5: Rate Limiting Global y Detección de DDoS
///
/// Rate limiter global que monitorea todas las conexiones
pub struct GlobalRateLimiter {
    /// Rate limiter por nodo
    per_node_limiter: RateLimiter,
    /// Rate limiter global por IP
    per_ip_limiter: RateLimiter,
    /// Historial de IPs para detección de DDoS
    ip_history: Mutex<HashMap<IpAddr, u32>>,
    /// Lista negra de IPs
    blacklist: Mutex<Vec<IpAddr>>,
    /// Umbral de detección de DDoS
    ddos_threshold: u32,
}

impl GlobalRateLimiter {
    /// Crea un nuevo rate limiter global
    pub fn new(per_node_rps: u32, global_rps: u32, ddos_threshold: u32) -> Self {
        Self {
            per_node_limiter: RateLimiter::new(per_node_rps),
            per_ip_limiter: RateLimiter::new(global_rps),
            ip_history: Mutex::new(HashMap::new()),
            blacklist: Mutex::new(Vec::new()),
            ddos_threshold,
        }
    }

    /// Verifica si una solicitud puede ser procesada
    /// Retorna (puede_procesar, es_ddos)
    pub fn check(&self, node_id: &[u8; 32], ip: IpAddr) -> (bool, bool) {
        // Verificar si está en blacklist
        let blacklist = self.blacklist.lock().unwrap();
        if blacklist.contains(&ip) {
            return (false, true);
        }
        drop(blacklist);

        // Verificar rate limit por nodo
        if !self.per_node_limiter.check(node_id) {
            return (false, false);
        }

        // Verificar rate limit global por IP
        if !self.per_ip_limiter.check(&ip_to_node_id(&ip)) {
            // Registrar el intento
            self.record_ip_attempt(&ip);
            return (false, false);
        }

        // Verificar si es un ataque DDoS potencial
        let is_ddos = self.is_ddos_attack(&ip);

        (true, is_ddos)
    }

    /// Registra un intento de conexión por IP
    fn record_ip_attempt(&self, ip: &IpAddr) {
        let mut history = self.ip_history.lock().unwrap();
        let count = history.entry(*ip).or_insert(0);
        *count += 1;
    }

    /// Determina si hay un ataque DDoS en progreso
    fn is_ddos_attack(&self, ip: &IpAddr) -> bool {
        let history = self.ip_history.lock().unwrap();
        if let Some(count) = history.get(ip) {
            *count > self.ddos_threshold
        } else {
            false
        }
    }

    /// Agrega una IP a la lista negra
    pub fn blacklist_ip(&self, ip: IpAddr) {
        let mut blacklist = self.blacklist.lock().unwrap();
        if !blacklist.contains(&ip) {
            blacklist.push(ip);
        }

        // Limpiar historial de esa IP
        let mut history = self.ip_history.lock().unwrap();
        history.remove(&ip);
    }

    /// Remueve una IP de la lista negra
    pub fn unblacklist_ip(&self, ip: IpAddr) {
        let mut blacklist = self.blacklist.lock().unwrap();
        blacklist.retain(|x| *x != ip);
    }

    /// Limpia el historial de IPs
    pub fn cleanup_history(&self) {
        let mut history = self.ip_history.lock().unwrap();
        history.clear();
    }

    /// Obtiene el número de IPs bloqueadas
    pub fn blocked_ip_count(&self) -> usize {
        self.blacklist.lock().unwrap().len()
    }
}

/// Convierte una IP a un NodeID de 32 bytes para rate limiting
fn ip_to_node_id(ip: &IpAddr) -> [u8; 32] {
    let mut result = [0u8; 32];
    match ip {
        IpAddr::V4(ipv4) => {
            let octets = ipv4.octets();
            result[..4].copy_from_slice(&octets);
        }
        IpAddr::V6(ipv6) => {
            let octets = ipv6.octets();
            result[..16].copy_from_slice(&octets);
        }
    }
    result
}

pub struct SecurityEngine {
    pub identity: NodeIdentity,
    pub node_nonce: u64, // El nonce hallado vía PoW
    pub path_finder: PathFinder,
    pub fragmenter: FragmentationManager,
    pub circuit_manager: CircuitManager,
    /// ✅ FASE 2.5: Rate limiter global
    pub global_rate_limiter: Option<GlobalRateLimiter>,
}

impl SecurityEngine {
    pub fn new(routing_table: RoutingTable) -> Self {
        let identity = NodeIdentity::generate();
        // Simulamos el minado del ID (En prod esto tardaría segundos)
        let node_nonce = 0; // Se asume minado previo o se llama a ProofOfWork::mine_id

        Self {
            identity,
            node_nonce,
            path_finder: PathFinder::new(routing_table),
            fragmenter: FragmentationManager::new(),
            circuit_manager: CircuitManager::new(),
            global_rate_limiter: None, // Inicializar sin rate limiter global
        }
    }

    /// ✅ FASE 2.5: Crea un SecurityEngine con rate limiting global
    pub fn new_with_rate_limiting(
        routing_table: RoutingTable,
        per_node_rps: u32,
        global_rps: u32,
        ddos_threshold: u32,
    ) -> Self {
        let identity = NodeIdentity::generate();
        let node_nonce = 0;

        Self {
            identity,
            node_nonce,
            path_finder: PathFinder::new(routing_table),
            fragmenter: FragmentationManager::new(),
            circuit_manager: CircuitManager::new(),
            global_rate_limiter: Some(GlobalRateLimiter::new(
                per_node_rps,
                global_rps,
                ddos_threshold,
            )),
        }
    }

    /// Prepara un paquete para ser enviado a través de la red seleccionando una ruta automática.
    pub fn protect_traffic_auto(&self, raw_data: Vec<u8>, hops: usize) -> Result<AAMNPacket> {
        // 1. Seleccionar ruta probabilística
        let path = self.path_finder.find_probabilistic_path(hops)?;

        // 2. Extraer claves y NodeIDs (en un sistema real las claves se derivan vía Handshake)
        // Para este prototipo, simulamos claves compartidas deterministas basadas en el ID del nodo
        let mut route_keys = Vec::new();
        let mut node_ids = Vec::new();

        for node in &path {
            route_keys.push(node.id); // Simulación de clave compartida
            node_ids.push(node.id);
        }

        // 3. Cifrado Onion
        let encrypted_payload = OnionEncryptor::wrap(&raw_data, &route_keys, &node_ids)?;

        // 4. Encapsulación y Padding
        let packet = AAMNPacket::new(encrypted_payload, 0).apply_padding();

        Ok(packet)
    }

    pub fn protect_traffic(
        &self,
        raw_data: Vec<u8>,
        route_keys: &[[u8; 32]],
        node_ids: &[[u8; 32]],
    ) -> Result<AAMNPacket> {
        // 1. Cifrado Onion
        let encrypted_payload = OnionEncryptor::wrap(&raw_data, route_keys, node_ids)?;

        // 2. Encapsulación y Padding
        let packet = AAMNPacket::new(encrypted_payload, 0) // fragment_id simplificado
            .apply_padding();

        Ok(packet)
    }

    /// Genera paquetes de ruido (Chaff Traffic) para mitigar la correlación de tráfico.
    /// Estos paquetes viajan por la red pero contienen datos aleatorios inservibles.
    pub fn generate_noise_packet(&self) -> Result<AAMNPacket> {
        let mut rng = rand::thread_rng();
        let noise_len = rng.gen_range(100..500);
        let mut noise_data = vec![0u8; noise_len];
        rand::thread_rng().fill(&mut noise_data[..]);

        // Los enviamos por una ruta aleatoria para confundir al observador estadístico
        self.protect_traffic_auto(noise_data, 3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing::NodeProfile;
    use chrono::Utc;

    #[test]
    fn test_full_onion_cycle() -> Result<()> {
        let mut table = RoutingTable::new();
        // Añadimos nodos para que el motor pueda inicializarse
        table.update_node(NodeProfile {
            id: [0u8; 32],
            endpoint: "127.0.0.1:8000".to_string(),
            last_seen: Utc::now(),
            latency_ms: 10,
            bandwidth_kbps: 1000,
            reputation: 1.0,
            staked_amount: 500,
            is_guard: true,
        });

        let _engine = SecurityEngine::new(table);
        let original_data = b"Mensaje Ultra Secreto AAMN".to_vec();

        // Simulamos una ruta de 3 nodos
        let key1 = [1u8; 32];
        let key2 = [2u8; 32];
        let key3 = [3u8; 32];

        let node1 = [11u8; 32];
        let node2 = [22u8; 32];
        let node3 = [33u8; 32]; // Nodo de salida o destino final

        let keys = vec![key1, key2, key3];
        let nodes = vec![node1, node2, node3];

        // Cifrar (Client side) - usar wrap directamente para evitar padding
        let encrypted_payload = OnionEncryptor::wrap(&original_data, &keys, &nodes)?;

        // Simular procesamiento en Node 1
        let (next_node1, payload1) = OnionEncryptor::unwrap(&encrypted_payload, &key1)?;
        assert_eq!(next_node1, node1);

        // Simular procesamiento en Node 2
        let (next_node2, payload2) = OnionEncryptor::unwrap(&payload1, &key2)?;
        assert_eq!(next_node2, node2);

        // Simular procesamiento en Node 3
        let (next_node3, payload3) = OnionEncryptor::unwrap(&payload2, &key3)?;
        assert_eq!(next_node3, node3);

        // El payload final debe contener los datos originales
        assert!(payload3.starts_with(&original_data));

        Ok(())
    }
}
