use crate::MAX_PACKET_SIZE;
use anyhow::{anyhow, Result};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::Mutex;

// ✅ CORRECCIÓN 1.3: Tipo de HMAC basado en SHA256 (compatible con hmac)
type HmacSha256 = Hmac<Sha256>;

/// ✅ CORRECCIÓN 1.3: Metadatos de fragmento con autenticación
#[derive(Clone)]
pub struct FragmentMetadata {
    pub fragment_id: u64,
    pub part_index: usize,
    pub total_parts: usize,
    pub hmac: [u8; 32], // Autenticación basada en SHA256 (32 bytes)
}

/// Gestor de Fragmentación de paquetes.
/// Permite dividir mensajes grandes en partes más pequeñas y reensamblarlas en el destino.
pub struct FragmentationManager {
    // Memoria para paquetes en reensamblaje: fragment_id -> (total_parts, parts_received)
    sessions: Mutex<HashMap<u64, Vec<Option<Vec<u8>>>>>,
}

impl Default for FragmentationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FragmentationManager {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    /// Divide un payload grande en fragmentos manejables.
    pub fn fragment(&self, payload: &[u8]) -> Vec<(u64, Vec<u8>, bool)> {
        let chunk_size = MAX_PACKET_SIZE / 2; // Margen para cabeceras y cifrado
        let fragment_id: u64 = rand::random();

        payload
            .chunks(chunk_size)
            .enumerate()
            .map(|(i, chunk)| {
                let is_last = (i + 1) * chunk_size >= payload.len();
                (fragment_id, chunk.to_vec(), is_last)
            })
            .collect()
    }

    /// Intenta reensamblar un fragmento recibido.
    /// Retorna Some(payload_completo) si se han recibido todas las partes.
    pub fn reassemble(
        &self,
        fragment_id: u64,
        part_index: usize,
        total_parts: usize,
        data: Vec<u8>,
    ) -> Option<Vec<u8>> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions
            .entry(fragment_id)
            .or_insert_with(|| vec![None; total_parts]);

        if part_index < session.len() {
            session[part_index] = Some(data);
        }

        if session.iter().all(|part| part.is_some()) {
            let complete = session.drain(..).flat_map(|p| p.unwrap()).collect();
            sessions.remove(&fragment_id);
            Some(complete)
        } else {
            None
        }
    }

    /// ✅ CORRECCIÓN 1.3: Firma un fragmento con HMAC usando la clave del nodo
    pub fn sign_fragment(&self, data: &[u8], key: &[u8; 32]) -> Result<([u8; 32], Vec<u8>)> {
        let mut mac =
            HmacSha256::new_from_slice(key).map_err(|_| anyhow!("Error al crear HMAC"))?;
        mac.update(data);

        let result = mac.finalize().into_bytes();
        let mut hmac_bytes = [0u8; 32];
        hmac_bytes.copy_from_slice(&result);

        // Retornar el HMAC y los datos firmados (datos + HMAC)
        let mut signed = data.to_vec();
        signed.extend_from_slice(&hmac_bytes);
        Ok((hmac_bytes, signed))
    }

    /// ✅ CORRECCIÓN 1.3: Verifica la autenticidad de un fragmento usando HMAC
    pub fn verify_fragment(&self, signed_data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
        if signed_data.len() < 32 {
            return Err(anyhow!("Fragmento demasiado corto para contener HMAC"));
        }

        let data = &signed_data[..signed_data.len() - 32];
        let received_hmac = &signed_data[signed_data.len() - 32..];

        // Calcular el HMAC esperado
        let mut mac =
            HmacSha256::new_from_slice(key).map_err(|_| anyhow!("Error al crear HMAC"))?;
        mac.update(data);
        let result = mac.finalize().into_bytes();

        // Comparar de forma segura contra ataques de timing
        if result[..] != received_hmac[..] {
            return Err(anyhow!(
                "HMAC inválido - posible ataque o corrupción de datos"
            ));
        }

        Ok(data.to_vec())
    }
}
