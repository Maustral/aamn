# 🛡️ AAMN Internal Security Audit & Cryptographic Specification (v1.0)

This document outlines the security architecture and self-audit findings for the **Adaptive Anonymous Mesh Network (AAMN)** as of version 0.5/1.0.

## 1. Cryptographic Primitive Layer

| Component | Primitive | Purpose |
| :--- | :--- | :--- |
| **Identity** | Ed25519 | Node IDs, Peer Authentication |
| **Key Exchange** | X25519 (ECDH) | Session key establishment per hop |
| **Symmetric Encryption** | ChaCha20-Poly1305 | AEAD for hop-by-hop onion decryption |
| **Integrity** | BLAKE3 / HMAC-SHA256 | Internal cell integrity checks |
| **Entropy** | OsRng (Fortuna-based) | Cryptographically secure random number generation |

## 2. Onion Routing Architecture

AAMN implements a **3-to-5 hop source-routed circuit model**:

1.  **Circuit Construction:** The source node selects $N$ nodes from the DHT (prioritizing high-reputation Guard nodes for the first hop).
2.  **Layered Encryption:** Data is wrapped in $N$ layers of encryption. Each layer is keyed with a unique session key established via Diffie-Hellman.
3.  **Perfect Forward Secrecy (PFS):** Session keys are rotated every $M$ packets or $T$ minutes (configurable).
4.  **Anti-Tagging:** Every cell is padded to a constant size (fixed-length cells) to prevent traffic analysis via packet size.

## 3. Threat Model & Mitigations

| Threat | Mitigation Strategy | Status |
| :--- | :--- | :--- |
| **Global Passive Observer** | **Bandwidth Shaping:** Constant-rate cover traffic (Chaff Injection) obscures peaks in actual usage. | ✅ Implemented |
| **Sybil Attack** | **Node Staking & Reputation:** Nodes must commit "stake" (simulation) and maintain uptime to gain DHT visibility. | ✅ Implemented |
| **Traffic Confirmation** | **Timing Perturbation:** Nodes introduce jitter in relaying packets to decorrelate ingress/egress timing. | 🛡️ v1.1 Planned |
| **Exit Node Logging** | **End-to-End TLS:** SOCKS5 users are encouraged to use HTTPS/TLS; AAMN adds the anonymization layer. | ✅ User Guidance |

## 4. Codebase Audit Checklist

- [x] **Zero-Copy Fragments:** Are we avoiding unnecessary copies of sensitive data? *Yes, using byte offsets and efficient buffer management.*
- [x] **Memory Safety:** Are `unsafe` blocks used? *Minimal. Only where absolutely necessary for low-level network performance, properly guarded.*
- [x] **Constant Time Specs:** Are cryptographic comparisons constant-time? *Used `subtle` crate or standard crypto libraries that guarantee constant-time ops.*
- [x] **Dependency Analysis:** Check for supply chain vulnerabilities. *Dependencies pinned and audited for v0.5 release.*

## 5. Deployment Security (Docker)

The provided `docker-compose.yml` uses:
- Non-privileged execution (where possible).
- Isolated networks for dashboard-to-dashboard communication.
- Volume pinning to persist node identities (preventing identity rotation on restart which looks like a Sybil attack).

---
**Approval Status:** 
The AAMN protocol version 1.0 architecture is considered **Production Ready** for experimental use cases. Continuous monitoring via the integrated **gRPC Dashboard** is recommended during initial rollout.
