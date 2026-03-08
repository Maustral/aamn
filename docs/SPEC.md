# RFC: AAMN Protocol Specification v1.0

## Abstract
This document specifies the Adaptive Anonymous Mesh Network (AAMN) protocol, a layered encryption (Onion Routing) system designed for low-latency, resilient, and anonymous communication over heterogeneous mesh networks.

## 1. Introduction
AAMN provides a source-routed, circuit-switched anonymity layer. It differs from traditional Tor circuits by incorporating adaptive node selection based on real-time reputation and network performance metrics.

## 2. Packet Format (Cells)
Every AAMN cell is exactly 512 bytes to prevent traffic analysis.

| Offset | Length | Description |
| :--- | :--- | :--- |
| 0 | 16 | Cell IV / Nonce |
| 16 | 32 | HMAC-SHA256 (Integrity) |
| 48 | 464 | Layer-encrypted Payload |

## 3. Cryptography
- **Asymmetric**: Ed25519 (Identity), X25519 (Ephemeral DH).
- **Symmetric**: ChaCha20-Poly1305.
- **Hashing**: BLAKE3.

## 4. Node Types
- **Guard Nodes**: Entry points with high uptime and staking requirements.
- **Relay Nodes**: Intermediate hops in the circuit.
- **Exit Nodes**: Nodes bridging AAMN to the public internet.

## 5. Circuit Establishment (The Onion Handshake)
1. Alice retrieves NodeProfiles from the DHT.
2. Alice performs an n-stage DH handshake (source-routing):
   - Wrap $PK_{Alice}$ for Node 3.
   - Encapsulate for Node 2.
   - Encapsulate for Node 1.
3. Each node peels its layer and forwards the inner payload.

## 6. Stability Policy
AAMN covers the Control API (gRPC/REST) with a Semantic Versioning guarantee.
- v1.x will maintain backward compatibility for all `proto/control.proto` endpoints.
- Breaking changes require a major version bump and a migration guide.
