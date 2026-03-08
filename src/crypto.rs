use anyhow::{anyhow, Result};
use blake2::{Blake2b512, Digest};
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit, Nonce};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
pub use x25519_dalek::PublicKey as X25519PublicKey;
use x25519_dalek::StaticSecret;

/// ✅ FASE 3: Funciones de protección de memoria
///
/// Trait para数据类型 que pueden ser limpiados de memoria
pub trait SecureZero {
    /// Limpia la memoria estableciendo todos los bytes a cero
    fn secure_zero(&mut self);
}

impl SecureZero for Vec<u8> {
    /// Limpia el vector de bytes estableciendo todos los valores a cero
    fn secure_zero(&mut self) {
        // Usar volatile write para evitar optimizaciones del compilador
        for byte in self.iter_mut() {
            unsafe {
                std::ptr::write_volatile(byte, 0);
            }
        }
        self.clear();
        self.shrink_to(0);
    }
}

impl SecureZero for [u8; 32] {
    /// Limpia el array de 32 bytes
    fn secure_zero(&mut self) {
        for byte in self.iter_mut() {
            unsafe {
                std::ptr::write_volatile(byte, 0);
            }
        }
    }
}

impl SecureZero for [u8; 64] {
    /// Limpia el array de 64 bytes
    fn secure_zero(&mut self) {
        for byte in self.iter_mut() {
            unsafe {
                std::ptr::write_volatile(byte, 0);
            }
        }
    }
}

/// ✅ FASE 3: Estructura de clave que limpia memoria automáticamente
/// Envuelve una clave y limpia la memoria cuando seDrop
pub struct SecureKey(pub [u8; 32]);

impl Drop for SecureKey {
    fn drop(&mut self) {
        self.secure_zero();
    }
}

impl SecureZero for SecureKey {
    fn secure_zero(&mut self) {
        for byte in self.0.iter_mut() {
            unsafe {
                std::ptr::write_volatile(byte, 0);
            }
        }
    }
}

impl Clone for SecureKey {
    fn clone(&self) -> Self {
        SecureKey(self.0)
    }
}

/// ✅ ÁREAS DE MEJORA: HKDF, Nonce Tracking, Key Derivation
use std::collections::HashSet;
use std::sync::Mutex;

/// ✅ Implementación de HKDF usando la librería `hkdf`
pub struct Hkdf {
    /// Clave maestra para derivación
    master_key: [u8; 32],
}

impl Hkdf {
    /// Crea un nuevo HKDF con una clave maestra
    pub fn new(master_key: &[u8; 32]) -> Self {
        Self {
            master_key: *master_key,
        }
    }

    /// Deriva una subclave usando HKDF-SHA256
    pub fn derive_key(&self, info: &[u8], output_len: usize) -> Vec<u8> {
        use hkdf::Hkdf;
        use sha2::Sha256;

        let hk = Hkdf::<Sha256>::new(Some(info), &self.master_key);
        let mut okm = vec![0u8; output_len];
        hk.expand(&[], &mut okm).expect("HKDF expand failed");
        okm
    }

    /// Deriva una clave de 32 bytes
    pub fn derive_session_key(&self, context: &str) -> [u8; 32] {
        let mut key = [0u8; 32];
        let derived = self.derive_key(context.as_bytes(), 32);
        key.copy_from_slice(&derived);
        key
    }
}

/// ✅ Nonce Tracker para prevenir reuse de nonces
pub struct NonceTracker {
    /// Conjunto de nonces usados (para detectar replay attacks)
    used_nonces: Mutex<HashSet<[u8; 12]>>,
    /// Máximo de nonces a rastrear
    max_tracked: usize,
}

impl NonceTracker {
    /// Crea un nuevo tracker
    pub fn new(max_tracked: usize) -> Self {
        Self {
            used_nonces: Mutex::new(HashSet::new()),
            max_tracked,
        }
    }

    /// Verifica si un nonce ya fue usado
    pub fn is_used(&self, nonce: &[u8; 12]) -> bool {
        let nonces = self.used_nonces.lock().unwrap();
        nonces.contains(nonce)
    }

    /// Registra un nonce como usado
    pub fn mark_used(&self, nonce: &[u8; 12]) -> bool {
        let mut nonces = self.used_nonces.lock().unwrap();

        // Limpiar si excede el máximo
        if nonces.len() >= self.max_tracked {
            // Eliminar la mitad más antigua
            let to_remove = nonces.len() / 2;
            let keys_to_remove: Vec<[u8; 12]> = nonces.iter().take(to_remove).cloned().collect();
            for key in keys_to_remove {
                nonces.remove(&key);
            }
        }

        nonces.insert(*nonce)
    }

    /// Limpia todos los nonces
    pub fn clear(&self) {
        let mut nonces = self.used_nonces.lock().unwrap();
        nonces.clear();
    }

    /// Retorna el número de nonces rastreados
    pub fn count(&self) -> usize {
        self.used_nonces.lock().unwrap().len()
    }
}

/// ✅ KDF Estándar para derivación de claves
pub struct KeyDerivationFunction {
    /// Nonce tracker para esta sesión
    nonce_tracker: NonceTracker,
}

impl KeyDerivationFunction {
    /// Crea un nuevo KDF
    pub fn new() -> Self {
        Self {
            nonce_tracker: NonceTracker::new(10000),
        }
    }

    /// Verifica y marca un nonce
    pub fn verify_and_mark_nonce(&self, nonce: &[u8; 12]) -> Result<bool, &'static str> {
        if self.nonce_tracker.is_used(nonce) {
            return Err("Nonce ya usado - posible ataque de replay");
        }

        if self.nonce_tracker.mark_used(nonce) {
            Ok(true)
        } else {
            Err("Error marcando nonce")
        }
    }

    /// Deriva una clave de sesión usando un KDF estándar
    pub fn derive_session_key(
        &self,
        master_secret: &[u8; 32],
        nonce: &[u8; 12],
        context: &[u8],
    ) -> [u8; 32] {
        use hkdf::Hkdf;
        use sha2::Sha256;

        // Combinar master secret + nonce + context
        let mut input = Vec::with_capacity(32 + 12 + context.len());
        input.extend_from_slice(master_secret);
        input.extend_from_slice(nonce);
        input.extend_from_slice(context);

        let hk = Hkdf::<Sha256>::new(None, &input);
        let mut okm = [0u8; 32];
        hk.expand(b"session-key-v1", &mut okm)
            .expect("HKDF expand failed");
        okm
    }
}

impl Default for KeyDerivationFunction {
    fn default() -> Self {
        Self::new()
    }
}

/// Representa la identidad de un nodo en la red AAMN.
pub struct NodeIdentity {
    pub signing_key: SigningKey,
    pub exchange_secret: StaticSecret,
}

impl NodeIdentity {
    pub fn generate() -> Self {
        let mut rng = OsRng;
        // En ed25519_dalek 2.x, SigningKey::generate toma una referencia mutable a un RNG
        #[allow(unused_mut)]
        let mut random_bytes = [0u8; 32];
        use rand::RngCore;
        rng.fill_bytes(&mut random_bytes);

        let signing_key = SigningKey::from_bytes(&random_bytes);
        let exchange_secret = StaticSecret::random_from_rng(rng);

        Self {
            signing_key,
            exchange_secret,
        }
    }

    pub fn public_id(&self) -> [u8; 32] {
        self.signing_key.verifying_key().to_bytes()
    }

    /// ✅ CORRECCIÓN 1.2: Deriva clave compartida con otro nodo usando X25519 (Diffie-Hellman)
    pub fn derive_shared_secret(&self, their_public_key: &X25519PublicKey) -> [u8; 32] {
        // Realizar intercambio Diffie-Hellman
        let shared_secret = self.exchange_secret.diffie_hellman(their_public_key);

        // Derivar clave de sesión usando HKDF con BLAKE2b
        let mut hasher = Blake2b512::new();
        hasher.update(shared_secret.as_bytes());
        let result = hasher.finalize();

        // Extraer los primeros 32 bytes como clave de sesión
        let mut key = [0u8; 32];
        key.copy_from_slice(&result[..32]);
        key
    }
}

/// Implementación de Cifrado Multicapa (Onion Encryption)
pub struct OnionEncryptor;

impl OnionEncryptor {
    /// Envuelve un payload en múltiples capas de cifrado.
    /// Cada capa corresponde a un nodo en la ruta.
    pub fn wrap(
        payload: &[u8],
        keys: &[[u8; 32]], // Lista de claves compartidas para cada salto (R1, R2, R3...)
        next_hops: &[[u8; 32]], // Lista de NodeIDs de los siguentes saltos
    ) -> Result<Vec<u8>> {
        let mut current_payload = payload.to_vec();
        let mut rng = OsRng; // ✅ CORRECCIÓN: Usar OsRng para generar nonces aleatorios

        // Envolvemos de atrás hacia adelante (capa más interna a la más externa)
        for (key_bytes, next_node) in keys.iter().zip(next_hops.iter()).rev() {
            let key = Key::from_slice(key_bytes);
            let cipher = ChaCha20Poly1305::new(key);

            // Creamos una estructura de control para este salto
            let mut layer_data = Vec::with_capacity(32 + current_payload.len());
            layer_data.extend_from_slice(next_node);
            layer_data.extend_from_slice(&current_payload);

            // ✅ CORRECCIÓN: Generar nonce aleatorio en lugar de derivarlo deterministicamente
            let mut nonce_bytes = [0u8; 12];
            use rand::RngCore;
            rng.fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);

            current_payload = cipher
                .encrypt(nonce, layer_data.as_ref())
                .map_err(|_| anyhow!("Error en cifrado onion"))?;

            // Adjuntamos el nonce para que el receptor pueda descifrar
            let mut final_layer = nonce_bytes.to_vec();
            final_layer.extend_from_slice(&current_payload);
            current_payload = final_layer;
        }

        Ok(current_payload)
    }

    /// Desenvuelve una capa de cifrado.
    pub fn unwrap(wrapped_payload: &[u8], shared_key: &[u8; 32]) -> Result<([u8; 32], Vec<u8>)> {
        if wrapped_payload.len() < 12 {
            return Err(anyhow!("Paquete demasiado corto"));
        }

        let nonce_bytes = &wrapped_payload[..12];
        let encrypted_data = &wrapped_payload[12..];

        let key = Key::from_slice(shared_key);
        let cipher = ChaCha20Poly1305::new(key);
        let nonce = Nonce::from_slice(nonce_bytes);

        let decrypted = cipher
            .decrypt(nonce, encrypted_data)
            .map_err(|_| anyhow!("Fallo al descifrar capa onion. ¿Clave incorrecta?"))?;

        if decrypted.len() < 32 {
            return Err(anyhow!("Payload descifrado corrupto"));
        }

        let mut next_node = [0u8; 32];
        next_node.copy_from_slice(&decrypted[..32]);
        let next_payload = decrypted[32..].to_vec();

        Ok((next_node, next_payload))
    }
}
