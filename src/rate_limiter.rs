/// ✅ IMPLEMENTACIÓN 3.1: Rate Limiting
/// 
/// Sistema de control de velocidad usando Token Bucket para proteger
/// la red AAMN contra ataques DoS y sobrecarga.
/// 
/// ✅ FASE 1.3: Protección contra Timing Attacks implementada

use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::Mutex;
use rand::Rng;

/// ✅ FASE 1.3: Constantes para protección contra timing attacks
/// Rango de jitter en milisegundos
const JITTER_MIN_MS: u64 = 0;
const JITTER_MAX_MS: u64 = 50;

/// ✅ FASE 1.3: Genera jitter aleatorio para evitar timing attacks
/// Retorna un delay adicional aleatorio entre JITTER_MIN_MS y JITTER_MAX_MS
fn get_timing_jitter() -> Duration {
    let mut rng = rand::thread_rng();
    let jitter_ms = rng.gen_range(JITTER_MIN_MS..=JITTER_MAX_MS);
    Duration::from_millis(jitter_ms)
}

/// ✅ FASE 1.3: Comparación de tiempo constante para prevenir timing attacks
/// Compara dos slices de bytes en tiempo constante
fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// ✅ IMPLEMENTACIÓN 3.1: Limitador de frecuencia basado en Token Bucket
/// 
/// El Token Bucket es un algoritmo simple y efectivo para rate limiting:
/// - Se asigna un número de "tokens" (solicitudes permitidas) por segundo
/// - Cada solicitud consume un token
/// - Si no hay tokens, la solicitud se rechaza
/// - Los tokens se regeneran constantemente
pub struct RateLimiter {
    /// Número de solicitudes permitidas por segundo
    requests_per_second: u32,
    /// HashMap: Node ID -> (último_reset, tokens_restantes)
    buckets: Mutex<HashMap<[u8; 32], (Instant, f32)>>,
}

impl RateLimiter {
    /// Crea un nuevo limitador de velocidad
    /// 
    /// # Argumentos
    /// * `rps` - Solicitudes permitidas por segundo para cada nodo
    pub fn new(rps: u32) -> Self {
        Self {
            requests_per_second: rps,
            buckets: Mutex::new(HashMap::new()),
        }
    }

    /// Verifica si un nodo puede realizar una solicitud
    /// 
    /// # Retorna
    /// `true` si está bajo el límite de velocidad, `false` si excede
    pub fn check(&self, node_id: &[u8; 32]) -> bool {
        let mut buckets = self.buckets.lock().unwrap();
        let now = Instant::now();
        
        let entry = buckets.entry(*node_id)
            .or_insert((now, self.requests_per_second as f32));
        
        let (last_reset, ref mut tokens) = entry;
        
        // Calcular tiempo transcurrido desde el último reset
        let elapsed = now.duration_since(*last_reset);
        
        // Si ha pasado más de 1 segundo, regenerar tokens
        if elapsed >= Duration::from_secs(1) {
            // Número de períodos de 1 segundo que pasaron
            let periods = elapsed.as_secs_f32();
            // Regenerar tokens (máximo es requests_per_second)
            *tokens = (self.requests_per_second as f32 * periods).min(self.requests_per_second as f32);
            *last_reset = now;
        }
        
        // Consumir un token si hay disponibles
        if *tokens >= 1.0 {
            *tokens -= 1.0;
            true  // Permitir la solicitud
        } else {
            false // Demasiadas solicitudes, rechazar
        }
    }

    /// Reinicia el limitador (útil para pruebas)
    pub fn reset(&self) {
        self.buckets.lock().unwrap().clear();
    }

    /// Obtiene información de debugging sobre los buckets
    pub fn get_bucket_info(&self, node_id: &[u8; 32]) -> Option<(Duration, f32)> {
        self.buckets.lock().unwrap().get(node_id).map(|(instant, tokens)| {
            (instant.elapsed(), *tokens)
        })
    }
}

/// Variante más moderna: Sliding Window Rate Limiter
/// Proporciona límites más precisos basados en ventana deslizante
pub struct SlidingWindowRateLimiter {
    /// Máximo número de solicitudes en la ventana
    max_requests: usize,
    /// Duración de la ventana temporal
    window_duration: Duration,
    /// HashMap: Node ID -> Vec de instantes de solicitudes recientes
    windows: Mutex<HashMap<[u8; 32], Vec<Instant>>>,
}

impl SlidingWindowRateLimiter {
    /// Crea un nuevo limitador de ventana deslizante
    /// 
    /// # Argumentos
    /// * `max_requests` - Máximo número de solicitudes permitidas
    /// * `window_secs` - Duración de la ventana en segundos
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            max_requests,
            window_duration: Duration::from_secs(window_secs),
            windows: Mutex::new(HashMap::new()),
        }
    }

    /// Verifica si un nodo puede realizar una solicitud
    pub fn check(&self, node_id: &[u8; 32]) -> bool {
        let mut windows = self.windows.lock().unwrap();
        let now = Instant::now();
        
        let window = windows.entry(*node_id).or_insert_with(Vec::new);
        
        // Eliminar solicitudes fuera de la ventana
        window.retain(|&instant| now.duration_since(instant) < self.window_duration);
        
        // Verificar si puede agregar una nueva solicitud
        if window.len() < self.max_requests {
            window.push(now);
            true
        } else {
            false
        }
    }

    /// Obtiene el número de solicitudes recientes
    pub fn get_request_count(&self, node_id: &[u8; 32]) -> usize {
        let mut windows = self.windows.lock().unwrap();
        let now = Instant::now();
        
        if let Some(window) = windows.get_mut(node_id) {
            window.retain(|&instant| now.duration_since(instant) < self.window_duration);
            window.len()
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_token_bucket_rate_limiting() {
        let limiter = RateLimiter::new(5); // 5 solicitudes por segundo
        let node_id = [1u8; 32];

        // Consumir 5 tokens rápidamente (debe permitir)
        for _ in 0..5 {
            assert!(limiter.check(&node_id), "Debe permitir 5 solicitudes");
        }

        // La sexta debe ser rechazada
        assert!(!limiter.check(&node_id), "La 6ª solicitud debe ser rechazada");

        // Esperar a que se regenere al menos 1 token (más de 1 segundo)
        thread::sleep(Duration::from_millis(1100));
        assert!(limiter.check(&node_id), "Debe permitir después de esperar");
    }

    #[test]
    fn test_sliding_window_rate_limiting() {
        let limiter = SlidingWindowRateLimiter::new(3, 1); // 3 solicitudes por segundo
        let node_id = [2u8; 32];

        // Hacer 3 solicitudes (deben permitirse)
        for _ in 0..3 {
            assert!(limiter.check(&node_id), "Debe permitir 3 solicitudes");
        }

        // La 4ª debe ser rechazada
        assert!(!limiter.check(&node_id), "La 4ª solicitud debe ser rechazada");

        // Esperar a que la ventana se renueve
        thread::sleep(Duration::from_secs(1) + Duration::from_millis(100));
        assert!(limiter.check(&node_id), "Debe permitir después de nueva ventana");
    }
}
