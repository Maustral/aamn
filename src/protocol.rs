use serde::{Serialize, Deserialize};
use crate::MAX_PACKET_SIZE;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AAMNPacket {
    pub version: u8,
    pub route_entropy: u32,
    pub ttl: u8,
    pub fragment_id: u64,
    pub payload: Vec<u8>,
}

impl AAMNPacket {
    pub fn new(payload: Vec<u8>, fragment_id: u64) -> Self {
        use rand::{thread_rng, Rng};
        let mut rng = thread_rng();
        
        Self {
            version: crate::PROTOCOL_VERSION,
            route_entropy: rng.gen(),
            ttl: 64,
            fragment_id,
            payload,
        }
    }
    
    /// Valida el paquete para verificar que es válido
    pub fn validate(&self) -> Result<(), &'static str> {
        // Verificar versión
        if self.version != crate::PROTOCOL_VERSION {
            return Err("Versión de protocolo inválida");
        }
        
        // Verificar TTL
        if self.ttl == 0 {
            return Err("TTL expirado");
        }
        
        // Verificar tamaño máximo
        if self.payload.len() > MAX_PACKET_SIZE {
            return Err("Payload demasiado grande");
        }
        
        // Verificar que no hay overflow en fragment_id
        if self.fragment_id > u64::MAX / 2 {
            return Err("Fragment ID inválido");
        }
        
        Ok(())
    }

    /// Aplica padding para que todos los paquetes tengan un tamaño uniforme.
    /// Esto evita el análisis de longitud por parte de adversarios en la red.
    pub fn apply_padding(mut self) -> Self {
        let current_len = bincode::serialize(&self).unwrap_or_default().len();
        if current_len < MAX_PACKET_SIZE {
            let padding_len = MAX_PACKET_SIZE - current_len;
            let padding = vec![0u8; padding_len];
            self.payload.extend_from_slice(&padding);
        }
        self
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ControlMessage {
    HandshakeInit {
        public_key: [u8; 32],
        ephemeral_key: [u8; 32],
    },
    HandshakeResponse {
        ephemeral_key: [u8; 32],
        tag: [u8; 16],
    },
    KeepAlive,
    RouteUpdate {
        nodes: Vec<[u8; 32]>,
    }
}
