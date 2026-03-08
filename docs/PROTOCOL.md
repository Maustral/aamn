# AAMN Protocol Specification

## Overview

AAMN (Adaptive Anonymous Mesh Network) is a privacy-enhancing network protocol designed to provide anonymous communication through a mix network architecture. The protocol implements onion routing with probabilistic path selection, fragmentation, and layered encryption.

**⚠️ WARNING**: This is a research prototype. Do not use for real communications without a security audit.

---

## Table of Contents

1. [Protocol Version](#protocol-version)
2. [Network Model](#network-model)
3. [Packet Format](#packet-format)
4. [Onion Encryption](#onion-encryption)
5. [Path Selection](#path-selection)
6. [Fragmentation](#fragmentation)
7. [Transport Layer](#transport-layer)
8. [Handshake Protocol](#handshake-protocol)
9. [Security Considerations](#security-considerations)

---

## 1. Protocol Version

Current version: `1`

```rust
pub const PROTOCOL_VERSION: u8 = 1;
```

The protocol version is included in packet headers for version negotiation.

---

## 2. Network Model

### 2.1 Node Types

| Type | Description | Entry/Exit |
|------|-------------|------------|
| Entry Node | First hop in circuit | Entry only |
| Middle Node | Relay nodes | Both |
| Exit Node | Final hop to destination | Exit only |

### 2.2 Network Topology

```
Client ----> Entry ----> Middle ----> Exit ----> Destination
              Node         Node         Node
              
              Circuit (3 hops minimum)
```

### 2.3 Node Identity

Each node has a cryptographic identity:

- **Signing Key**: Ed25519 for authentication
- **Exchange Secret**: X25519 for key exchange
- **Node ID**: SHA-256 hash of public key (32 bytes)

```rust
pub struct NodeIdentity {
    pub signing_key: SigningKey,      // Ed25519
    pub exchange_secret: StaticSecret, // X25519
}

impl NodeIdentity {
    pub fn public_id(&self) -> [u8; 32] {
        // SHA-256 of verifying key
    }
}
```

---

## 3. Packet Format

### 3.1 AAMN Packet Structure

```
+----------------+----------------+----------------+
| Header (16B)   | Payload (var)  | Padding (var)  |
+----------------+----------------+----------------+

Header:
  - version: u8 (1 byte)
  - fragment_id: u64 (8 bytes)
  - flags: u8 (1 byte)
  - reserved: u48 (6 bytes)
```

### 3.2 Encrypted Payload (Onion)

```
Before Encryption (per layer):
+----------------+----------------+----------------+
| Next Node (32B)| Payload        | HMAC (32B)     |
+----------------+----------------+----------------+

After Encryption:
+----------------+----------------+
| Nonce (12B)    | Ciphertext     |
+----------------+----------------+
```

### 3.3 Fragment Format

```
+----------------+----------------+----------------+
| Frag ID (8B)   | Index (2B)     | Total (2B)     |
+----------------+----------------+----------------+
| Data (var)     | HMAC (32B)     |
+----------------+----------------+
```

---

## 4. Onion Encryption

### 4.1 Layer Structure

Each layer contains:
1. **Next Node ID**: 32 bytes - Where to forward
2. **Inner Payload**: The encrypted inner layers
3. **Layer MAC**: HMAC-SHA256 for authenticity

### 4.2 Encryption Process

```rust
fn wrap(payload, keys, next_hops) -> wrapped {
    // Process from innermost to outermost
    for (key, next_node) in reversed(zip(keys, next_hops)) {
        layer_data = next_node + current_payload
        nonce = random(12 bytes)
        ciphertext = ChaCha20Poly1305(key, nonce, layer_data)
        current_payload = nonce + ciphertext
    }
    return current_payload
}
```

### 4.3 Decryption Process

```rust
fn unwrap(wrapped, key) -> (next_node, inner) {
    nonce = wrapped[0:12]
    ciphertext = wrapped[12:]
    plaintext = ChaCha20Poly1305_decrypt(key, nonce, ciphertext)
    next_node = plaintext[0:32]
    inner = plaintext[32:]
    return (next_node, inner)
}
```

### 4.4 Cipher Suite

- **Encryption**: ChaCha20-Poly1305
- **Key Derivation**: X25519 + HKDF (BLAKE2b)
- **Authentication**: HMAC-SHA256
- **Nonce**: Random 12 bytes (per message)

---

## 5. Path Selection

### 5.1 Probabilistic Selection

Path selection uses weighted random sampling:

```rust
fn select_node(available_nodes, weights) -> Node {
    // Weighted random selection based on:
    // - Bandwidth (higher = more likely)
    // - Reputation (higher = more likely)
    // - Latency (lower = more likely)
    // - Stake amount (higher = more likely)
}
```

### 5.2 Path Constraints

| Parameter | Minimum | Maximum | Default |
|-----------|---------|---------|---------|
| Path Length | 2 | 5 | 3 |
| Entry Node Selection | - | - | Top 20% bandwidth |
| Exit Node Selection | - | - | Any reputable |

### 5.3 Circuit Rotation

Circuits should be rotated:
- Every 10 minutes (time-based)
- After 1GB of traffic (volume-based)
- On node failure

---

## 6. Fragmentation

### 6.1 Fragmentation Strategy

- **Maximum Fragment Size**: 512 bytes (configurable)
- **Fragment ID**: Unique identifier for reassembly
- **Total Parts**: Number of fragments

### 6.2 Reassembly

Fragments are reassembled using:
1. Fragment ID matching
2. Index ordering
3. Total parts verification
4. HMAC verification

### 6.3 Padding

Each fragment is padded to uniform size to prevent traffic analysis.

---

## 7. Transport Layer

### 7.1 QUIC Protocol

The transport layer uses QUIC (UDP-based):

```rust
pub struct TransportLayer {
    pub endpoint: quinn::Endpoint,
}

// Connection establishment
let conn = transport.connect(addr).await?;

// Send packet
transport.send_packet(&conn, data).await?;

// Receive packets
while let Some(packet) = transport.listen_packets(&conn).await {
    // Process
}
```

### 7.2 Port Allocation

- **Default Port**: 9000
- **Ephemeral Range**: 49152-65535

---

## 8. Handshake Protocol

### 8.1 Noise Protocol IKpsk2

The handshake uses Noise Protocol Framework with pre-shared keys:

```
Initiator -> Responder: e, es (ephemeral key, static DH)
Responder -> Initiator: e, ee, s, se (DH operations)
Initiator -> Responder: s, ss (finalize)
```

### 8.2 Key Derivation

```
HKDF(Secret, Info) -> Session Keys
```

### 8.3 Handshake Messages

```rust
pub struct HandshakeOutput {
    pub handshake_message: Vec<u8>,
    pub noise_state: AnyState,
}

pub struct HandshakeResponse {
    pub response_message: Vec<u8>,
    pub verified: bool,
}
```

---

## 9. Security Considerations

### 9.1 Threat Model

The protocol protects against:
- ✅ Traffic analysis
- ✅ End-to-end correlation
- ✅ Node compromise (partial)
- ✅ Passive adversaries

The protocol does NOT protect against:
- ⚠️ Active man-in-the-middle
- ⚠️ Node compromise (full circuit)
- ⚠️ Global passive adversary
- ⚠️ Timing attacks (without additional countermeasures)

### 9.2 Attack Mitigations

| Attack | Mitigation |
|--------|------------|
| Traffic Analysis | Onion encryption, padding, chaff traffic |
| Route Correlation | Multiple paths, random selection |
| Node Tracing | Layered encryption, HMAC |
| DoS | Rate limiting, PoW |

### 9.3 Chaff Traffic

To prevent statistical traffic analysis, nodes inject random packets (chaff):

```rust
// Probability: 10% (configurable)
if random() < config.chaff_probability {
    generate_noise_packet()
    send_via_alternative_route()
}
```

---

## 10. Protocol Messages

### 10.1 Message Types

| Type | ID | Description |
|------|----|-------------|
| DATA | 0x01 | Encrypted data payload |
| CONNECT | 0x02 | Connection request |
| CONNECTED | 0x03 | Connection established |
| DISCONNECT | 0x04 | Disconnection request |
| PING | 0x05 | Keepalive |
| PONG | 0x06 | Keepalive response |
| ERROR | 0xFF | Error message |

### 10.2 Error Codes

| Code | Description |
|------|-------------|
| 0x01 | Invalid packet |
| 0x02 | Decryption failed |
| 0x03 | Route not found |
| 0x04 | Node unavailable |
| 0x05 | Rate limit exceeded |

---

## 11. Implementation Notes

### 11.1 Dependencies

```toml
# Cryptography
chacha20poly1305 = "0.10"
x25519-dalek = "2.0"
ed25519-dalek = "2.1"
blake2 = "0.10"
hmac = "0.12"

# Networking
quinn = "0.10"
tokio = "1.36"

# Utilities
anyhow = "1.0"
serde = "1.0"
```

### 11.2 Performance Targets

| Metric | Target |
|--------|--------|
| Latency (3-hop) | < 500ms |
| Throughput | > 10 Mbps |
| Memory per node | < 100 MB |
| CPU per packet | < 10 ms |

---

## 12. References

- [Tor Protocol Specification](https://gitweb.torproject.org/torspec.git)
- [Noise Protocol Framework](https://noiseprotocol.org/)
- [I2P Book](https://geti2p.net/spec)
- [QUIC Protocol](https://www.rfc-editor.org/rfc/rfc9000)

---

*Document Version: 1.0*  
*Last Updated: 2025*

