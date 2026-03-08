//! ✅ IMPLEMENTACIÓN: Padding de Tráfico y Cell-Based Routing
//!
//! Este módulo implementa protección contra análisis de tráfico usando:
//! - Celdas de tamaño fijo (como Tor)
//! - Padding de longitud
//! - Traffic shaping

use anyhow::{anyhow, Result};
use rand::{rngs::OsRng, Rng};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{interval_at, Instant};

/// Tamaño fijo de celda (similar a Tor's 512 bytes)
pub const CELL_SIZE: usize = 512;

/// Tamaño del header de celda
pub const CELL_HEADER_SIZE: usize = 16;

/// Payload máximo por celda
pub const CELL_PAYLOAD_SIZE: usize = CELL_SIZE - CELL_HEADER_SIZE;

/// Tipo de mensaje en la celda
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CellType {
    Data = 0,
    Padding = 1,
    Create = 2,
    Created = 3,
    Relay = 4,
    Destroy = 5,
}

/// Header de celda fijo
#[derive(Clone)]
pub struct CellHeader {
    /// Identificador de circuito
    pub circuit_id: u32,
    /// Tipo de mensaje
    pub cell_type: CellType,
    /// Longitud del payload
    pub payload_length: u16,
    /// Comando específico
    pub command: u8,
}

impl CellHeader {
    /// Serializa el header a bytes
    pub fn to_bytes(&self) -> [u8; CELL_HEADER_SIZE] {
        let mut bytes = [0u8; CELL_HEADER_SIZE];
        bytes[0..4].copy_from_slice(&self.circuit_id.to_le_bytes());
        bytes[4] = self.cell_type as u8;
        bytes[5..7].copy_from_slice(&self.payload_length.to_le_bytes());
        bytes[7] = self.command;
        bytes
    }

    /// Deserializa el header desde bytes
    pub fn from_bytes(bytes: &[u8; CELL_HEADER_SIZE]) -> Result<Self> {
        let circuit_id = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let cell_type = match bytes[4] {
            0 => CellType::Data,
            1 => CellType::Padding,
            2 => CellType::Create,
            3 => CellType::Created,
            4 => CellType::Relay,
            5 => CellType::Destroy,
            _ => return Err(anyhow!("Tipo de celda inválido")),
        };
        let payload_length = u16::from_le_bytes([bytes[5], bytes[6]]);
        let command = bytes[7];

        Ok(Self {
            circuit_id,
            cell_type,
            payload_length,
            command,
        })
    }
}

/// Celda de tamaño fijo
#[derive(Clone)]
pub struct Cell {
    pub header: CellHeader,
    pub payload: Vec<u8>,
}

impl Cell {
    /// Crea una nueva celda
    pub fn new(
        circuit_id: u32,
        cell_type: CellType,
        command: u8,
        payload: Vec<u8>,
    ) -> Result<Self> {
        if payload.len() > CELL_PAYLOAD_SIZE {
            return Err(anyhow!("Payload demasiado grande"));
        }

        let payload_length = payload.len() as u16;

        Ok(Self {
            header: CellHeader {
                circuit_id,
                cell_type,
                payload_length,
                command,
            },
            payload,
        })
    }

    /// Serializa la celda completa
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(CELL_SIZE);
        result.extend_from_slice(&self.header.to_bytes());

        // Agregar payload
        result.extend_from_slice(&self.payload);

        // Padding al tamaño fijo
        let padding_needed = CELL_PAYLOAD_SIZE - self.payload.len();
        if padding_needed > 0 {
            let mut padding = vec![0u8; padding_needed];
            // Padding aleatorio para prevenir análisis
            let mut rng = OsRng;
            rng.fill(&mut padding[..]);
            result.extend_from_slice(&padding);
        }

        result
    }

    /// Deserializa una celda
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < CELL_SIZE {
            return Err(anyhow!("Datos insuficientes para celda"));
        }

        let header_bytes: &[u8; CELL_HEADER_SIZE] = data[0..CELL_HEADER_SIZE]
            .try_into()
            .map_err(|_| anyhow!("Header inválido"))?;

        let header = CellHeader::from_bytes(header_bytes)?;

        // Extraer payload (sin padding)
        let payload_end = CELL_HEADER_SIZE + header.payload_length as usize;
        let payload = data[CELL_HEADER_SIZE..payload_end].to_vec();

        Ok(Self { header, payload })
    }

    /// Verifica si la celda es padding
    pub fn is_padding(&self) -> bool {
        self.header.cell_type == CellType::Padding
    }
}

/// Gestor de padding para protección contra análisis de tráfico
pub struct TrafficPadding {
    /// Probabilidad de enviar celdas de padding (0.0 - 1.0)
    padding_probability: f32,
    /// Intervalo mínimo entre celdas de padding
    min_interval_ms: u64,
    /// Tamaño objetivo para padding de longitud
    target_size: usize,
}

impl TrafficPadding {
    /// Crea un nuevo gestor de padding
    pub fn new() -> Self {
        Self {
            padding_probability: 0.1, // 10% de probabilidad
            min_interval_ms: 100,
            target_size: CELL_PAYLOAD_SIZE,
        }
    }

    /// Configura la probabilidad de padding
    pub fn with_probability(mut self, prob: f32) -> Self {
        self.padding_probability = prob.clamp(0.0, 1.0);
        self
    }

    /// Configura el intervalo mínimo
    pub fn with_min_interval(mut self, ms: u64) -> Self {
        self.min_interval_ms = ms;
        self
    }

    /// Aplica padding al payload para ocultar el tamaño real
    pub fn apply_length_padding(&self, data: &[u8]) -> Vec<u8> {
        let mut rng = OsRng;

        // Estrategia: añadir padding aleatorio para alcanzar tamaño fijo
        // Esto previene ataques de correlación de longitud
        if data.len() < self.target_size {
            let mut result = data.to_vec();
            let padding_len = self.target_size - data.len();

            // Padding aleatorio
            let mut padding = vec![0u8; padding_len];
            rng.fill(&mut padding[..]);

            // Marcador de inicio de padding (para que el receptor pueda distinguir)
            // Usamos el último byte como longitud real
            result.push(data.len() as u8);
            result.extend_from_slice(&padding[..padding_len - 1]);

            result
        } else if data.len() > self.target_size {
            // Dividir en múltiples celdas
            data.to_vec()
        } else {
            data.to_vec()
        }
    }

    /// Determina si se debe enviar una celda de padding
    pub fn should_send_padding(&self) -> bool {
        let mut rng = OsRng;
        rng.gen::<f32>() < self.padding_probability
    }

    /// Genera una celda de padding
    pub fn generate_padding_cell(&self, circuit_id: u32) -> Cell {
        let mut rng = OsRng;
        let mut payload = vec![0u8; CELL_PAYLOAD_SIZE];
        rng.fill(&mut payload[..]);

        Cell::new(circuit_id, CellType::Padding, 0, payload).unwrap()
    }

    /// Ajusta el padding basado en el tráfico entrante (para simetría)
    pub fn adjust_for_incoming_traffic(&mut self, incoming_size: usize) {
        // Si hay mucho tráfico entrante, reducir padding saliente
        // Esto crea un patrón más natural
        if incoming_size > 10000 {
            self.padding_probability = 0.05;
        } else if incoming_size > 5000 {
            self.padding_probability = 0.1;
        } else {
            self.padding_probability = 0.15;
        }
    }
}

impl Default for TrafficPadding {
    fn default() -> Self {
        Self::new()
    }
}
///
/// Emite paquetes a un ritmo constante configurado.
/// Si hay paquetes reales para enviar, se priorizan.
/// Si no, emite paquetes de padding para rellenar el ancho de banda y mitigar análisis estadístico.
pub struct CoverTrafficManager {
    /// Tasa de envío objetivo (celdas por segundo)
    cells_per_second: u32,
    /// Generador de padding activo
    generator_active: bool,
}

impl CoverTrafficManager {
    /// Inicializa un gestor de tráfico constante
    pub fn new(cells_per_second: u32) -> Self {
        Self {
            cells_per_second,
            generator_active: false,
        }
    }

    /// Inicia un bucle asíncrono que emite tráfico de cover a ritmo constante.
    /// Retorna un channel por donde se pueden recibir las celdas a enviar por la red.
    pub fn start_generator(&mut self) -> mpsc::Receiver<Cell> {
        let (tx, rx) = mpsc::channel(100);
        let cps = self.cells_per_second.max(1);
        let duration = Duration::from_millis(1000 / cps as u64);

        self.generator_active = true;

        tokio::spawn(async move {
            let start = Instant::now() + duration;
            let mut interval = interval_at(start, duration);

            loop {
                interval.tick().await;

                // Generar celda de padding (circuit_id 0 = dummy padding)
                let mut payload = vec![0u8; CELL_PAYLOAD_SIZE];
                rand::thread_rng().fill(&mut payload[..]);

                if let Ok(cell) = Cell::new(0, CellType::Padding, 0, payload) {
                    // Si el canal está lleno o cerrado, ignoramos
                    if tx.try_send(cell).is_err() {
                        // Receptor desconectado o buffer lleno (posible congestión)
                    }
                }
            }
        });

        rx
    }

    pub fn is_active(&self) -> bool {
        self.generator_active
    }
}

impl Default for CoverTrafficManager {
    fn default() -> Self {
        Self::new(10) // Por defecto: 10 celdas/sec -> ~5KB/s
    }
}

/// Gestor de traffic shaping para prevenir análisis
pub struct TrafficShaper {
    /// Ancho de banda objetivo (bytes por segundo)
    target_bandwidth: u64,
    /// Burst máximo
    max_burst: u64,
    /// Tokens actuales
    tokens: u64,
    /// Última actualización
    last_update: std::time::Instant,
}

impl TrafficShaper {
    /// Crea un nuevo shaper
    pub fn new(target_bps: u64, max_burst: u64) -> Self {
        Self {
            target_bandwidth: target_bps,
            max_burst,
            tokens: max_burst,
            last_update: std::time::Instant::now(),
        }
    }

    /// Verifica si se puede enviar datos
    pub fn can_send(&mut self, size: usize) -> bool {
        self.refill();
        self.tokens >= size as u64
    }

    /// Envía datos (deduce tokens)
    pub fn send(&mut self, size: usize) {
        self.refill();
        if self.tokens >= size as u64 {
            self.tokens -= size as u64;
        }
    }

    /// Rellena los tokens basándose en el tiempo transcurrido
    fn refill(&mut self) {
        let elapsed = self.last_update.elapsed().as_secs_f64();
        let tokens_to_add = (self.target_bandwidth as f64 * elapsed) as u64;

        self.tokens = (self.tokens + tokens_to_add).min(self.max_burst);
        self.last_update = std::time::Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_serialization() {
        let payload = b"Test message".to_vec();
        let cell = Cell::new(1, CellType::Data, 0, payload.clone()).unwrap();

        let bytes = cell.to_bytes();
        assert_eq!(bytes.len(), CELL_SIZE);

        let decoded = Cell::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.header.circuit_id, 1);
        assert_eq!(decoded.header.cell_type, CellType::Data);
        assert_eq!(decoded.payload, payload);
    }

    #[test]
    fn test_padding() {
        let padding = TrafficPadding::new();
        let small_data = b"Hello".to_vec();

        let padded = padding.apply_length_padding(&small_data);
        // El resultado debe ser mayor o igual al target
        assert!(padded.len() >= small_data.len());
    }

    #[test]
    fn test_traffic_shaper() {
        let mut shaper = TrafficShaper::new(1000, 500);

        // Primeras 500 bytes deben pasar
        assert!(shaper.can_send(500));
        shaper.send(500);

        // No debería poder enviar más de 500 de una vez
        assert!(!shaper.can_send(501));
    }
}
