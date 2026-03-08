//! Implementación de DHT (Kademlia) para descubrimiento de nodos en la red AAMN
//!
//! Kademlia es un protocolo de tabla de hash distribuida (DHT) usado por redes P2P
//! para el descubrimiento de pares sin necesidad de un servidor central.
//!
//! ✅ 2.5 CORRECCIÓN: Autenticación de mensajes implementada con HMAC

use anyhow::{anyhow, Result};
use hmac::{Hmac, Mac};
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Tipo HMAC para autenticación de mensajes DHT
type HmacSha256 = Hmac<Sha256>;

/// Tamaño del ID de nodo en bytes (256 bits)
pub const NODE_ID_SIZE: usize = 32;

/// Tamaño del bucket de Kademlia (k-buckets)
const K_BUCKET_SIZE: usize = 20;

/// Cantidad de nodos a solicitar en FIND_NODE
const ALPHA: usize = 3;

/// Representa un ID de nodo en la red Kademlia
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub [u8; NODE_ID_SIZE]);

impl NodeId {
    /// Genera un nuevo ID de nodo aleatorio
    pub fn generate() -> Self {
        let mut id = [0u8; NODE_ID_SIZE];
        let mut rng = OsRng;
        rng.fill_bytes(&mut id);
        Self(id)
    }

    /// Crea un ID a partir de bytes
    pub fn from_bytes(bytes: [u8; NODE_ID_SIZE]) -> Self {
        Self(bytes)
    }

    /// Calcula la distancia XOR entre dos nodos
    pub fn distance(&self, other: &NodeId) -> [u8; NODE_ID_SIZE] {
        let mut result = [0u8; NODE_ID_SIZE];
        for i in 0..NODE_ID_SIZE {
            result[i] = self.0[i] ^ other.0[i];
        }
        result
    }

    /// Calcula la distancia XOR como entero para comparaciones
    pub fn distance_as_u128(&self, other: &NodeId) -> u128 {
        let dist = self.distance(other);
        u128::from_be_bytes([
            dist[0], dist[1], dist[2], dist[3], dist[4], dist[5], dist[6], dist[7], dist[8],
            dist[9], dist[10], dist[11], dist[12], dist[13], dist[14], dist[15],
        ])
    }
}

/// Información de un nodo en la red
#[derive(Clone, Debug)]
pub struct NodeInfo {
    pub id: NodeId,
    pub address: SocketAddr,
    pub last_seen: Instant,
    pub last_pinged: Option<Instant>,
    pub failed_pings: u32,
}

impl NodeInfo {
    pub fn new(id: NodeId, address: SocketAddr) -> Self {
        Self {
            id,
            address,
            last_seen: Instant::now(),
            last_pinged: None,
            failed_pings: 0,
        }
    }

    pub fn is_stale(&self, timeout: Duration) -> bool {
        Instant::now().duration_since(self.last_seen) > timeout
    }
}

/// Bucket de Kademlia que almacena hasta k nodos
#[derive(Clone)]
pub struct KBucket {
    nodes: Vec<NodeInfo>,
    min_distance: u8, // Distancia mínima (índice del bit)
    max_distance: u8, // Distancia máxima
}

impl KBucket {
    pub fn new(min_distance: u8, max_distance: u8) -> Self {
        Self {
            nodes: Vec::with_capacity(K_BUCKET_SIZE),
            min_distance,
            max_distance,
        }
    }

    pub fn add(&mut self, node: NodeInfo) -> Option<NodeInfo> {
        // Si el nodo ya existe, actualizar su información
        if let Some(pos) = self.nodes.iter().position(|n| n.id == node.id) {
            let old = self.nodes[pos].clone();
            self.nodes[pos] = node;
            return Some(old);
        }

        // Si hay espacio, agregar
        if self.nodes.len() < K_BUCKET_SIZE {
            self.nodes.push(node);
            return None;
        }

        // Bucket lleno - el más antiguo es el primero
        let oldest = self.nodes.remove(0);
        self.nodes.push(node);
        Some(oldest)
    }

    pub fn remove(&mut self, node_id: &NodeId) -> Option<NodeInfo> {
        if let Some(pos) = self.nodes.iter().position(|n| n.id == *node_id) {
            Some(self.nodes.remove(pos))
        } else {
            None
        }
    }

    pub fn get_nodes(&self) -> &[NodeInfo] {
        &self.nodes
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_full(&self) -> bool {
        self.nodes.len() >= K_BUCKET_SIZE
    }
}

/// Protocolo de mensajes DHT
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DhtMessageType {
    Ping = 0,
    Pong = 1,
    Store = 2,
    FindNode = 3,
    FindValue = 4,
    FindNodeResponse = 5,
    FindValueResponse = 6,
}

/// Mensaje del protocolo DHT
#[derive(Debug, Clone)]
pub struct DhtMessage {
    pub msg_type: DhtMessageType,
    pub transaction_id: [u8; 16],
    pub sender_id: NodeId,
    pub data: Vec<u8>,
}

impl DhtMessage {
    pub fn new(msg_type: DhtMessageType, sender_id: NodeId, data: Vec<u8>) -> Self {
        let mut transaction_id = [0u8; 16];
        let mut rng = OsRng;
        rng.fill_bytes(&mut transaction_id);

        Self {
            msg_type,
            transaction_id,
            sender_id,
            data,
        }
    }

    pub fn ping(sender_id: NodeId) -> Self {
        Self::new(DhtMessageType::Ping, sender_id, vec![])
    }

    pub fn pong(sender_id: NodeId) -> Self {
        Self::new(DhtMessageType::Pong, sender_id, vec![])
    }

    pub fn find_node(sender_id: NodeId, target_id: NodeId) -> Self {
        Self::new(DhtMessageType::FindNode, sender_id, target_id.0.to_vec())
    }

    pub fn find_value(sender_id: NodeId, key: [u8; 32]) -> Self {
        Self::new(DhtMessageType::FindValue, sender_id, key.to_vec())
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut result = vec![self.msg_type as u8];
        result.extend_from_slice(&self.transaction_id);
        result.extend_from_slice(&self.sender_id.0);

        // Agregar longitud de datos y datos
        let len = self.data.len() as u32;
        result.extend_from_slice(&len.to_le_bytes());
        result.extend_from_slice(&self.data);

        result
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < 53 {
            return Err(anyhow!(
                "Datos demasiado cortos para mensaje DHT (min 53 bytes)"
            ));
        }

        let msg_type = match data[0] {
            0 => DhtMessageType::Ping,
            1 => DhtMessageType::Pong,
            2 => DhtMessageType::Store,
            3 => DhtMessageType::FindNode,
            4 => DhtMessageType::FindValue,
            5 => DhtMessageType::FindNodeResponse,
            6 => DhtMessageType::FindValueResponse,
            _ => return Err(anyhow!("Tipo de mensaje desconocido")),
        };

        let mut transaction_id = [0u8; 16];
        transaction_id.copy_from_slice(&data[1..17]);

        let mut sender_id_bytes = [0u8; 32];
        sender_id_bytes.copy_from_slice(&data[17..49]);

        let mut len_bytes = [0u8; 4];
        len_bytes.copy_from_slice(&data[49..53]);
        let len = u32::from_le_bytes(len_bytes) as usize;

        if data.len() < 53 + len {
            return Err(anyhow!("Datos incompletos en mensaje DHT"));
        }

        let data_bytes = data[53..53 + len].to_vec();

        Ok(Self {
            msg_type,
            transaction_id,
            sender_id: NodeId(sender_id_bytes),
            data: data_bytes,
        })
    }

    // ==================== ✅ 2.5: AUTENTICACIÓN HMAC ====================

    /// ✅ 2.5: Firma el mensaje con HMAC usando la clave del nodo
    pub fn sign(&self, key: &[u8; 32]) -> Vec<u8> {
        let mut mac =
            HmacSha256::new_from_slice(key).expect("HMAC puede tomar clave de cualquier tamaño");

        // Incluir todos los campos relevantes en el HMAC
        mac.update(&[self.msg_type as u8]);
        mac.update(&self.transaction_id);
        mac.update(&self.sender_id.0);
        mac.update(&self.data);

        let result = mac.finalize().into_bytes();
        result.to_vec()
    }

    /// ✅ 2.5: Verifica la autenticidad del mensaje
    pub fn verify(&self, key: &[u8; 32], expected_hmac: &[u8]) -> bool {
        let computed_hmac = self.sign(key);
        // Comparación de tiempo constante para prevenir timing attacks
        if computed_hmac.len() != expected_hmac.len() {
            return false;
        }

        computed_hmac
            .iter()
            .zip(expected_hmac.iter())
            .fold(0u8, |acc, (a, b)| acc | (a ^ b))
            == 0
    }

    /// ✅ 2.5: Codifica el mensaje con HMAC adjunto
    pub fn encode_signed(&self, key: &[u8; 32]) -> Vec<u8> {
        let mut encoded = self.encode();
        let hmac = self.sign(key);
        encoded.extend_from_slice(&hmac);
        encoded
    }

    /// ✅ 2.5: Decodifica y verifica un mensaje con HMAC
    pub fn decode_verified(data: &[u8], key: &[u8; 32]) -> Result<Self> {
        if data.len() < 32 {
            return Err(anyhow!("Datos demasiado cortos"));
        }

        // Extraer HMAC (últimos 32 bytes)
        let hmac_size = 32;
        let msg_data = &data[..data.len() - hmac_size];
        let received_hmac = &data[data.len() - hmac_size..];

        // Decodificar el mensaje
        let msg = Self::decode(msg_data)?;

        // Verificar HMAC
        if !msg.verify(key, received_hmac) {
            return Err(anyhow!("HMAC inválido - mensaje manipulado"));
        }

        Ok(msg)
    }
}

/// Almacenamiento de valores (DHT)
pub struct DhtStorage {
    data: HashMap<[u8; 32], (Vec<u8>, Instant)>,
    expiration: Duration,
}

impl DhtStorage {
    pub fn new(expiration: Duration) -> Self {
        Self {
            data: HashMap::new(),
            expiration,
        }
    }

    pub fn store(&mut self, key: [u8; 32], value: Vec<u8>) {
        self.data.insert(key, (value, Instant::now()));
    }

    pub fn get(&self, key: &[u8; 32]) -> Option<Vec<u8>> {
        self.data.get(key).and_then(|(value, timestamp)| {
            if Instant::now().duration_since(*timestamp) < self.expiration {
                Some(value.clone())
            } else {
                None
            }
        })
    }

    pub fn remove(&mut self, key: &[u8; 32]) -> Option<Vec<u8>> {
        self.data.remove(key).map(|(v, _)| v)
    }

    pub fn cleanup(&mut self) {
        self.data.retain(|_, (_, timestamp)| {
            Instant::now().duration_since(*timestamp) < self.expiration
        });
    }
}

/// Routing table de Kademlia
pub struct KademliaRoutingTable {
    buckets: Vec<KBucket>,
    local_id: NodeId,
}

impl KademliaRoutingTable {
    pub fn new(local_id: NodeId) -> Self {
        // Crear 256 buckets (uno por cada bit del ID)
        let mut buckets = Vec::with_capacity(256);
        for i in 0..256 {
            buckets.push(KBucket::new(i as u8, i as u8));
        }

        Self { buckets, local_id }
    }

    pub fn local_id(&self) -> &NodeId {
        &self.local_id
    }

    /// Agrega un nodo a la tabla de routing
    pub fn add_node(&mut self, node: NodeInfo) -> Option<NodeInfo> {
        let distance = self.local_id.distance(&node.id);
        let bucket_index = self.get_bucket_index(&distance);

        if bucket_index < self.buckets.len() {
            self.buckets[bucket_index].add(node)
        } else {
            None
        }
    }

    /// Elimina un nodo de la tabla
    pub fn remove_node(&mut self, node_id: &NodeId) -> Option<NodeInfo> {
        let distance = self.local_id.distance(node_id);
        let bucket_index = self.get_bucket_index(&distance);

        if bucket_index < self.buckets.len() {
            self.buckets[bucket_index].remove(node_id)
        } else {
            None
        }
    }

    /// Encuentra los k nodos más cercanos al ID objetivo
    pub fn find_closest(&self, target_id: &NodeId, count: usize) -> Vec<NodeInfo> {
        let mut candidates: Vec<([u8; 32], NodeInfo)> = Vec::new();

        // Recolectar nodos de todos los buckets
        for bucket in &self.buckets {
            for node in bucket.get_nodes() {
                if node.id != self.local_id {
                    let dist = target_id.distance(&node.id);
                    candidates.push((dist, node.clone()));
                }
            }
        }

        // Ordenar por distancia
        candidates.sort_by(|a, b| {
            let dist_a = self.local_id.distance(&a.1.id);
            let dist_b = self.local_id.distance(&b.1.id);
            dist_a.cmp(&dist_b)
        });

        // Retornar los primeros 'count' nodos
        candidates
            .into_iter()
            .take(count)
            .map(|(_, info)| info)
            .collect()
    }

    /// Obtiene el índice del bucket basándose en la distancia
    fn get_bucket_index(&self, distance: &[u8; 32]) -> usize {
        for (i, &byte) in distance.iter().enumerate() {
            if byte != 0 {
                for bit in 0..8 {
                    if byte & (0x80 >> bit) != 0 {
                        return i * 8 + bit;
                    }
                }
            }
        }
        255 // Distancia máxima
    }

    /// Obtiene todos los nodos conocidos
    pub fn get_all_nodes(&self) -> Vec<NodeInfo> {
        self.buckets
            .iter()
            .flat_map(|b| b.get_nodes().to_vec())
            .collect()
    }

    /// Verifica si un nodo existe en la tabla
    pub fn contains(&self, node_id: &NodeId) -> bool {
        self.find_closest(node_id, K_BUCKET_SIZE)
            .iter()
            .any(|n| n.id == *node_id)
    }
}

/// Estado de la conexión con un nodo
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Failed,
}

/// Gestor principal del DHT
pub struct DhtManager {
    routing_table: Arc<RwLock<KademliaRoutingTable>>,
    storage: Arc<RwLock<DhtStorage>>,
    pending_requests: Arc<RwLock<HashMap<[u8; 16], tokio::sync::oneshot::Sender<DhtMessage>>>>,
    bootstrap_nodes: Vec<SocketAddr>,
}

impl DhtManager {
    /// Crea un nuevo gestor DHT
    pub fn new(local_id: NodeId, bootstrap_nodes: Vec<SocketAddr>) -> Self {
        Self {
            routing_table: Arc::new(RwLock::new(KademliaRoutingTable::new(local_id))),
            storage: Arc::new(RwLock::new(DhtStorage::new(Duration::from_secs(3600)))),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            bootstrap_nodes,
        }
    }

    /// Inicializa el DHT conectándose a nodos bootstrap
    pub async fn bootstrap(&self) -> Result<()> {
        for addr in &self.bootstrap_nodes {
            // Intentar conectar y hacer ping
            self.ping_node(addr).await?;
        }
        Ok(())
    }

    /// Envía un ping a un nodo
    pub async fn ping_node(&self, addr: &SocketAddr) -> Result<NodeInfo> {
        let local_id = self.routing_table.read().await.local_id().clone();

        // Simular respuesta (en implementación real, esto sería una llamada de red)
        // Por ahora, agregamos el nodo como reachable
        let node_info = NodeInfo::new(local_id.clone(), *addr);

        self.routing_table.write().await.add_node(node_info.clone());

        Ok(node_info)
    }

    /// Busca nodos cercanos a un ID objetivo
    pub async fn find_node(&self, target_id: NodeId) -> Result<Vec<NodeInfo>> {
        let closest = self
            .routing_table
            .read()
            .await
            .find_closest(&target_id, K_BUCKET_SIZE);

        if closest.is_empty() && !self.bootstrap_nodes.is_empty() {
            // Si no hay nodos, usar bootstrap
            for addr in &self.bootstrap_nodes {
                let node = NodeInfo::new(target_id.clone(), *addr);
                return Ok(vec![node]);
            }
        }

        Ok(closest)
    }

    /// Busca un valor en la DHT
    pub async fn find_value(&self, key: &[u8; 32]) -> Result<Option<Vec<u8>>> {
        // Primero buscar en almacenamiento local
        let local_value = self.storage.read().await.get(key);
        if let Some(value) = local_value {
            return Ok(Some(value));
        }

        // Buscar nodos que puedan tener el valor
        let target_id = NodeId(*key);
        let nodes = self.find_node(target_id).await?;

        // En implementación real, consultar nodos remotos
        // Por ahora, retornar el valor local si existe
        Ok(self.storage.read().await.get(key))
    }

    /// Almacena un valor en la DHT
    pub async fn store(&self, key: [u8; 32], value: Vec<u8>) -> Result<()> {
        // Almacenar localmente
        self.storage.write().await.store(key, value.clone());

        // Encontrar nodos cercanos para replicar
        let target_id = NodeId(key);
        let nodes = self.find_node(target_id).await?;

        // En implementación real, enviar STORE a nodos cercanos
        // Por ahora, solo almacenamos localmente
        Ok(())
    }

    /// Obtiene la tabla de routing
    pub async fn get_routing_table(&self) -> Vec<NodeInfo> {
        self.routing_table.read().await.get_all_nodes()
    }

    /// Obtiene el ID del nodo local
    pub async fn local_node_id(&self) -> NodeId {
        self.routing_table.read().await.local_id().clone()
    }

    /// Maneja un mensaje recibido
    pub async fn handle_message(&self, msg: DhtMessage) -> Option<DhtMessage> {
        match msg.msg_type {
            DhtMessageType::Ping => {
                let local_id = self.routing_table.read().await.local_id().clone();
                Some(DhtMessage::pong(local_id))
            }
            DhtMessageType::FindNode => {
                if msg.data.len() >= 32 {
                    let mut target_bytes = [0u8; 32];
                    target_bytes.copy_from_slice(&msg.data[..32]);
                    let target_id = NodeId(target_bytes);

                    let closest = self
                        .routing_table
                        .read()
                        .await
                        .find_closest(&target_id, K_BUCKET_SIZE);

                    // Serializar nodos encontrados
                    let mut response_data = Vec::new();
                    for node in closest {
                        response_data.extend_from_slice(&node.id.0);
                        // Agregar puerto (2 bytes) + IP (variable)
                        let addr_str = node.address.to_string();
                        let addr_bytes = addr_str.as_bytes();
                        response_data.extend_from_slice(&(addr_bytes.len() as u16).to_le_bytes());
                        response_data.extend_from_slice(addr_bytes);
                    }

                    let local_id = self.routing_table.read().await.local_id().clone();
                    Some(DhtMessage {
                        msg_type: DhtMessageType::FindNodeResponse,
                        transaction_id: msg.transaction_id,
                        sender_id: local_id,
                        data: response_data,
                    })
                } else {
                    None
                }
            }
            DhtMessageType::FindValue => {
                if msg.data.len() >= 32 {
                    let mut key = [0u8; 32];
                    key.copy_from_slice(&msg.data[..32]);

                    if let Some(value) = self.storage.read().await.get(&key) {
                        let local_id = self.routing_table.read().await.local_id().clone();
                        let mut response_data = vec![1]; // Found flag
                        response_data.extend_from_slice(&value);

                        Some(DhtMessage {
                            msg_type: DhtMessageType::FindValueResponse,
                            transaction_id: msg.transaction_id,
                            sender_id: local_id,
                            data: response_data,
                        })
                    } else {
                        // No encontrado, retornar nodos cercanos (manejamos FindNode aquí)
                        if msg.data.len() >= 32 {
                            let mut target_bytes = [0u8; 32];
                            target_bytes.copy_from_slice(&msg.data[..32]);
                            let target_id = NodeId(target_bytes);

                            let closest = self
                                .routing_table
                                .read()
                                .await
                                .find_closest(&target_id, K_BUCKET_SIZE);

                            let mut response_data = Vec::new();
                            for node in closest {
                                response_data.extend_from_slice(&node.id.0);
                                let addr_str = node.address.to_string();
                                let addr_bytes = addr_str.as_bytes();
                                response_data
                                    .extend_from_slice(&(addr_bytes.len() as u16).to_le_bytes());
                                response_data.extend_from_slice(addr_bytes);
                            }

                            let local_id = self.routing_table.read().await.local_id().clone();
                            Some(DhtMessage {
                                msg_type: DhtMessageType::FindNodeResponse,
                                transaction_id: msg.transaction_id,
                                sender_id: local_id,
                                data: response_data,
                            })
                        } else {
                            None
                        }
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Actualiza el último tiempo visto de un nodo
    pub async fn touch_node(&self, node_id: &NodeId) {
        let mut rt = self.routing_table.write().await;

        // Encontrar y actualizar el nodo
        let all_nodes = rt.get_all_nodes();
        if let Some(node) = all_nodes.iter().find(|n| n.id == *node_id) {
            let mut updated_node = node.clone();
            updated_node.last_seen = Instant::now();
            rt.add_node(updated_node);
        }
    }

    /// Limpia nodos stale de la tabla de routing
    pub async fn cleanup_stale_nodes(&self, timeout: Duration) {
        let mut rt = self.routing_table.write().await;

        // Obtener todos los nodos
        let all_nodes = rt.get_all_nodes();

        // Eliminar nodos stale
        for node in all_nodes {
            if node.is_stale(timeout) {
                rt.remove_node(&node.id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    #[test]
    fn test_node_id_generation() {
        let id1 = NodeId::generate();
        let id2 = NodeId::generate();

        assert_ne!(id1, id2);
    }

    #[test]
    fn test_xor_distance() {
        let id1 = NodeId::from_bytes([0u8; 32]);
        let id2 = NodeId::from_bytes([0xFFu8; 32]);

        let dist = id1.distance(&id2);

        assert_eq!(dist, [0xFFu8; 32]);
    }

    #[test]
    fn test_kbucket_operations() {
        let mut bucket = KBucket::new(0, 0);
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

        let node1 = NodeInfo::new(NodeId::generate(), addr);
        let node2 = NodeInfo::new(NodeId::generate(), addr);

        bucket.add(node1.clone());
        assert_eq!(bucket.len(), 1);

        bucket.add(node2);
        assert!(bucket.len() >= 2);

        // Verify nodes can be retrieved
        let nodes = bucket.get_nodes();
        assert_eq!(nodes.len(), 2);
        assert!(nodes.iter().any(|n| n.id == node1.id));
    }

    #[tokio::test]
    async fn test_dht_manager() {
        let local_id = NodeId::generate();
        let dht = DhtManager::new(local_id, vec![]);

        let found = dht.find_node(NodeId::generate()).await;
        assert!(found.is_ok());
    }

    #[test]
    fn test_message_encoding() {
        let local_id = NodeId::generate();
        let msg = DhtMessage::ping(local_id.clone());

        let encoded = msg.encode();
        let decoded = DhtMessage::decode(&encoded).unwrap();

        assert_eq!(decoded.msg_type, DhtMessageType::Ping);
        assert_eq!(decoded.sender_id, local_id);
    }
}
