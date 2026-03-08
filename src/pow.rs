use blake2::{Blake2b512, Digest};

/// Sistema de Proof-of-Work para mitigar ataques Sybil.
/// Requiere que un NodeID tenga un hash que cumpla con cierta dificultad.
pub struct ProofOfWork;

impl ProofOfWork {
    /// Genera un "nonce" que hace que HASH(public_key + nonce) empiece con N ceros.
    pub fn mine_id(public_key: &[u8; 32], difficulty: u32) -> u64 {
        let mut nonce: u64 = 0;
        let target = difficulty; // Número de bits en cero requeridos al inicio

        loop {
            let mut hasher = Blake2b512::new();
            hasher.update(public_key);
            hasher.update(nonce.to_le_bytes());
            let result = hasher.finalize();

            if Self::check_difficulty(&result, target) {
                return nonce;
            }
            nonce += 1;
        }
    }

    /// Verifica si un ID cumple con la prueba de trabajo.
    pub fn verify(public_key: &[u8; 32], nonce: u64, difficulty: u32) -> bool {
        let mut hasher = Blake2b512::new();
        hasher.update(public_key);
        hasher.update(nonce.to_le_bytes());
        let result = hasher.finalize();

        Self::check_difficulty(&result, difficulty)
    }

    fn check_difficulty(hash: &[u8], difficulty_bits: u32) -> bool {
        let bytes_to_check = (difficulty_bits / 8) as usize;
        let remaining_bits = difficulty_bits % 8;

        // Verificar si los primeros `bytes_to_check` bytes son 0
        for &item in hash.iter().take(bytes_to_check) {
            if item != 0 {
                return false;
            }
        }

        if remaining_bits > 0 {
            let mask = 0xFF << (8 - remaining_bits);
            if (hash[bytes_to_check] & mask) != 0 {
                return false;
            }
        }

        true
    }
}
