# AAMN Protocol Specification

This document describes the AAMN wire protocol, cryptographic design, and message formats.

---

## Overview

AAMN routes messages through a series of relay nodes (a **circuit**) such that:
- No relay knows both the sender and the final destination.
- All traffic is indistinguishable in size (fixed 512-byte cells).
- Session keys are ephemeral and provide forward secrecy.

---

## 1. Node Identity

Each node generates an **Ed25519** key pair at first run:

```
identity_key = Ed25519::generate()
node_id      = SHA-256(identity_key.public_bytes())
```

The `node_id` is a 32-byte value used as the Kademlia key for DHT routing.

---

## 2. Handshake — Noise IKpsk2

AAMN uses the **Noise IKpsk2** handshake pattern from the [Noise Protocol Framework](https://noiseprotocol.org/).

### Pattern: `Noise_IKpsk2_25519_ChaChaPoly_BLAKE2s`

```
Initiator                              Responder
─────────────────────────────────────────────────
e, es, s                →
                         ←  e, ee, se, psk
```

- `e` — Ephemeral DH key (X25519)
- `s` — Static identity key (X25519)
- `es`, `ee`, `se` — DH operations
- `psk` — Pre-shared key for extra authentication

**Result**: Two symmetric session keys:
- `send_key` — used by the initiator to encrypt outbound data
- `recv_key` — used by the initiator to decrypt inbound data

---

## 3. Onion Encryption

### Cell Structure (512 bytes fixed)

```
┌──────────────────────────────────┐
│ Header (4 bytes)                 │
│   circuit_id: u16               │
│   cell_type:  u8                │
│   flags:      u8                │
├──────────────────────────────────┤
│ Payload (up to 492 bytes)        │
│   [encrypted data]               │
├──────────────────────────────────┤
│ Padding (variable, random bytes) │
└──────────────────────────────────┘
Total: exactly 512 bytes
```

### Wrapping Algorithm

For a circuit of N relays `[R1, R2, ..., RN]` with session keys `[K1, K2, ..., KN]`:

```
layer_N   = Encrypt(K_N, payload)
layer_N-1 = Encrypt(K_{N-1}, layer_N)
...
layer_1   = Encrypt(K_1, layer_2)

send(R1, layer_1)
```

Each relay `Ri` decrypts its layer to reveal the next hop and forwards the remaining ciphertext.

### Cipher: `ChaCha20-Poly1305`

- **Key**: 256-bit session key
- **Nonce**: 96-bit, monotonically increasing per session
- **AAD**: Cell header bytes

---

## 4. DHT — Kademlia

AAMN uses a **Kademlia** distributed hash table for peer discovery.

### Node ID Distance

```
distance(A, B) = A XOR B   (bitwise, 32 bytes)
```

### k-Bucket Structure

Each node maintains 256 k-buckets (one per bit of the node ID space). Each bucket holds up to **K=20** peers sorted by last-seen time.

### Messages

| Type | Description |
|---|---|
| `PING` | Check if a peer is alive |
| `PONG` | Response to PING |
| `FIND_NODE` | Request k closest peers to a target ID |
| `FOUND_NODES` | Response with peer list |
| `STORE` | Store a value in the DHT |
| `FIND_VALUE` | Retrieve a value from the DHT |

### Message Wire Format

```
┌─────────────────────────────────────────┐
│ transaction_id : [u8; 16]  (16 bytes)  │
│ msg_type       : u8         (1 byte)   │
│ sender_id      : [u8; 32]  (32 bytes)  │
│ payload_len    : u32        (4 bytes)  │
│ payload        : [u8]                  │
│ hmac           : [u8; 32]  (32 bytes)  │
└─────────────────────────────────────────┘
Minimum length: 85 bytes
```

---

## 5. Transport — QUIC

All relay-to-relay communication uses **QUIC** (via the `quinn` crate) with:

- Self-signed TLS 1.3 certificates per node (via `rcgen`)
- Certificate pinning — nodes only accept peers whose certificate hash is registered in the DHT
- Unidirectional QUIC streams for one-way cell delivery

---

## 6. Rate Limiting

### Token Bucket

Each peer connection is subject to a token-bucket limiter:
- Capacity: 100 tokens
- Refill: 10 tokens/second
- Cost per packet: 1 token

### Sliding Window

A sliding 60-second window tracks request counts per `node_id`. Nodes exceeding the threshold are temporarily blocked.

---

## 7. Proof of Work (Spam Prevention)

To join the network, new nodes must solve a SHA-256 PoW challenge:

```
Find nonce such that:
SHA-256(node_id || nonce) starts with N zero bits
```

Default difficulty: N = 16 bits.

---

## Security Properties

| Property | Mechanism |
|---|---|
| Confidentiality | ChaCha20-Poly1305 per hop |
| Forward secrecy | Ephemeral X25519 in Noise handshake |
| Sender anonymity | Onion routing — relays only see adjacent hops |
| Traffic analysis resistance | Fixed 512-byte cells, rate limiting |
| Peer authentication | Ed25519 node identity + PSK |
| Replay protection | Monotonic AEAD nonces |

---

*For implementation details, see the source code in `src/` and the [API reference](API.md).*
