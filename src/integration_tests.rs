//! Tests de Integración - AAMN Network

#[cfg(test)]
mod tests {
    use crate::crypto::{NodeIdentity, OnionEncryptor, X25519PublicKey};
    use crate::dht::{DhtManager, NodeId};
    use crate::handshake::HandshakeManager;
    use crate::padding::{Cell, CellType, TrafficPadding, TrafficShaper};
    use crate::protocol::AAMNPacket;
    use crate::rate_limiter::RateLimiter;
    use std::time::Duration;

    fn get_test_psk() -> [u8; 32] {
        let mut psk = [0u8; 32];
        let psk_bytes = b"test-psk-integration";
        let len = psk_bytes.len().min(32);
        psk[..len].copy_from_slice(&psk_bytes[..len]);
        psk
    }

    // Test: Pipeline de cifrado completo
    #[tokio::test]
    async fn test_full_encryption_pipeline() {
        let alice = NodeIdentity::generate();
        let bob = NodeIdentity::generate();

        // Correct way to get public key from static secret in x25519-dalek 2.0
        let bob_public = X25519PublicKey::from(&bob.exchange_secret);
        let shared_key = alice.derive_shared_secret(&bob_public);

        let plaintext = b"Mensaje secreto";
        let keys = vec![shared_key];
        let nodes = vec![alice.public_id()];

        let encrypted = OnionEncryptor::wrap(plaintext, &keys, &nodes);
        assert!(encrypted.is_ok());
    }

    // Test: Handshake entre nodos
    #[tokio::test]
    async fn test_handshake_integration() {
        let psk = get_test_psk();
        let alice_mgr = HandshakeManager::new(&psk);
        let bob_mgr = HandshakeManager::new(&psk);

        let alice_public = alice_mgr.public_key();
        let bob_public = bob_mgr.public_key();

        let output = alice_mgr.initiate_handshake(&bob_public);
        assert!(
            output.is_ok(),
            "Alice failed to initiate: {:?}",
            output.err()
        );

        let response =
            bob_mgr.respond_to_handshake(&alice_public, &output.unwrap().handshake_message);
        assert!(
            response.is_ok(),
            "Bob failed to respond: {:?}",
            response.err()
        );
    }

    // Test: DHT storage y retrieval
    #[tokio::test]
    async fn test_dht_store_and_retrieve() {
        let local_id = NodeId::generate();
        let dht = DhtManager::new(local_id, vec![]);

        let key = [1u8; 32];
        let value = b"test value".to_vec();

        let store = dht.store(key, value.clone()).await;
        assert!(store.is_ok());

        let retrieved = dht.find_value(&key).await;
        assert!(retrieved.is_ok());
    }

    // Test: Cell-based padding
    #[test]
    fn test_cell_padding() {
        let padding = TrafficPadding::new();
        let data = b"Short message";
        let padded = padding.apply_length_padding(data);
        assert!(padded.len() >= data.len());
    }

    // Test: Traffic shaper
    #[test]
    fn test_traffic_shaper() {
        let mut shaper = TrafficShaper::new(1000, 500);
        assert!(shaper.can_send(100));
        shaper.send(100);
        assert!(!shaper.can_send(600));
    }

    // Test: Cell serialization
    #[test]
    fn test_cell_roundtrip() {
        let payload = b"Test payload".to_vec();
        let cell = Cell::new(1, CellType::Data, 0, payload);
        assert!(cell.is_ok());

        let cell = cell.unwrap();
        let bytes = cell.to_bytes();
        assert_eq!(bytes.len(), 512);

        let decoded = Cell::from_bytes(&bytes);
        assert!(decoded.is_ok());
    }

    // Test: Rate limiter
    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(10);
        let node_id = [1u8; 32];

        for _ in 0..10 {
            assert!(limiter.check(&node_id));
        }
        assert!(!limiter.check(&node_id));
    }

    // Test: Packet validation
    #[test]
    fn test_packet_validation() {
        let packet = AAMNPacket::new(b"test".to_vec(), 1);
        assert!(packet.validate().is_ok());

        let mut invalid = AAMNPacket::new(b"test".to_vec(), 1);
        invalid.ttl = 0;
        assert!(invalid.validate().is_err());
    }

    // Test: End-to-end
    #[tokio::test]
    async fn test_e2e_message() {
        let sender = NodeIdentity::generate();
        let relay = NodeIdentity::generate();

        let relay_public = X25519PublicKey::from(&relay.exchange_secret);
        let key = sender.derive_shared_secret(&relay_public);

        let keys = vec![key];
        let nodes = vec![relay.public_id()];

        let encrypted = OnionEncryptor::wrap(b"Hola", &keys, &nodes);
        assert!(encrypted.is_ok());
    }
}
