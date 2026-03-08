# AAMN Roadmap

This document tracks the planned development of the AAMN project.

> **Current version**: v0.2.0 — Core Protocol

---

## v0.2 — Core Protocol ✅ *(Released)*

| Feature | Status |
|---|---|
| Noise IKpsk2 handshake (X25519 + PSK) | ✅ Done |
| Multi-layer onion encryption (ChaCha20-Poly1305) | ✅ Done |
| Kademlia DHT peer discovery | ✅ Done |
| QUIC transport layer | ✅ Done |
| Fixed-cell padding (512 bytes) | ✅ Done |
| Token-bucket + sliding-window rate limiting | ✅ Done |
| CLI (`start`, `stop`, `status`) | ✅ Done |
| 60 unit + integration tests | ✅ Done |
| CI/CD (Linux, macOS, Windows) | ✅ Done |
| GitHub Pages landing page | ✅ Done |

---

## v0.3 — Network Hardening 🔨 *(In Progress)*

Goal: Make AAMN able to route real traffic across multiple live nodes with strong anonymity guarantees.

| Feature | Status |
|---|---|
| Full onion circuit across 3+ live hops | 🔄 In progress |
| Session key rotation every N packets | 📋 Planned |
| Guard node selection algorithm | 📋 Planned |
| Constant-rate cover traffic (bandwidth shaping) | 📋 Planned |
| NAT traversal via STUN/TURN | 📋 Planned |
| Reconnect logic and circuit failover | 📋 Planned |
| Improved bootstrap node handling | 📋 Planned |

---

## v0.4 — Ecosystem 🌐

Goal: Make AAMN usable by other applications and developers.

| Feature | Status |
|---|---|
| SOCKS5 proxy interface | 📋 Planned |
| gRPC control API | 📋 Planned |
| Web dashboard (metrics + peers) | 📋 Planned |
| Docker / OCI container image | 📋 Planned |
| Mobile library (iOS/Android via FFI) | 💡 Research |
| Python bindings | 💡 Research |

---

## v0.5 — Resilience 🛡️

Goal: Make the network robust against adversarial conditions.

| Feature | Status |
|---|---|
| Sybil attack resistance (PoW-gated join) | 📋 Planned |
| Eclipse attack mitigation | 📋 Planned |
| Tor-style hidden services | 💡 Research |
| Reputation/staking system for relays | 💡 Research |
| End-to-end latency optimization | 📋 Planned |

---

## v1.0 — Production 🚀

Goal: A stable, audited, production-ready release.

| Feature | Status |
|---|---|
| External cryptographic security audit | 📋 Planned |
| RFC-style protocol specification | 📋 Planned |
| Stable API guarantee (semver) | 📋 Planned |
| Public benchmark suite | 📋 Planned |
| 90%+ test coverage | 📋 Planned |
| Long-term support (LTS) branch | 📋 Planned |

---

## Legend

| Symbol | Meaning |
|---|---|
| ✅ | Completed |
| 🔄 | In progress |
| 📋 | Planned |
| 💡 | Under research / exploratory |

---

*Last updated: March 2026*
