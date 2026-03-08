# AAMN Security Model

## ⚠️ Important Disclaimer

**This is a research prototype for educational purposes.** 

DO NOT use this software for:
- Real communications requiring anonymity
- Financial transactions
- Sensitive data transmission
- Any purpose where security is critical

Before using for production, this codebase requires:
- Professional security audit
- Formal verification of cryptographic implementations
- Extensive testing
- Review by cryptography experts

---

## Table of Contents

1. [Threat Model](#threat-model)
2. [Security Properties](#security-properties)
3. [Cryptographic Design](#cryptographic-design)
4. [Attack Surface](#attack-surface)
5. [Mitigations](#mitigations)
6. [Known Limitations](#known-limitations)
7. [Recommendations](#recommendations)

---

## 1. Threat Model

### 1.1 Assumptions

We assume an adversary who:

| Capability | Description |
|------------|-------------|
| Passive | Can observe network traffic |
| Active | Can inject/modify traffic |
| Local | Can observe local network |
| Global | Can observe large portion of network |

### 1.2 Trust Model

```
┌─────────────────────────────────────────────────────────────┐
│                    TRUSTED COMPONENTS                       │
├─────────────────────────────────────────────────────────────┤
│  - Local node software (we run it)                         │
│  - Our own cryptographic keys                              │
│  - Configuration we provide                                │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                   UNTRUSTED COMPONENTS                      │
├─────────────────────────────────────────────────────────────┤
│  - All other nodes in the network                          │
│  - Network infrastructure                                  │
│  - Entry/exit nodes                                        │
│  - Timing information                                      │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Security Properties

### 2.1 What We Provide

| Property | Implementation | Strength |
|----------|---------------|----------|
| **Confidentiality** | Onion encryption | Strong |
| **Integrity** | HMAC-SHA256 | Strong |
| **Anonymity** | Multi-hop routing | Moderate |
| **Forward Secrecy** | Session keys | Weak* |
| **Deniability** | Not implemented | None |

*Forward secrecy is limited due to static key usage

### 2.2 What We Don't Provide

- **Sender anonymity**: Traffic can be traced to entry node
- **Recipient anonymity**: Exit node knows destination
- **Relationship anonymity**: Timing can correlate traffic
- **Location anonymity**: IP addresses visible to ISP

---

## 3. Cryptographic Design

### 3.1 Cipher Suite

| Primitive | Algorithm | Key Size | Notes |
|-----------|-----------|----------|-------|
| Encryption | ChaCha20-Poly1305 | 256-bit | Authenticated |
| Key Exchange | X25519 | 256-bit | ECDH |
| Signing | Ed25519 | 256-bit | EdDSA |
| Hashing | BLAKE2b | 512-bit | - |
| MAC | HMAC-SHA256 | 256-bit | Authentication |
| PRF | HKDF | - | Key derivation |

### 3.2 Key Hierarchy

```
┌─────────────────────────────────────────────┐
│           Node Identity Keys                │
│  ┌─────────────────────────────────────┐   │
│  │ Signing Key (Ed25519)               │   │
│  │ Exchange Secret (X25519)            │   │
│  └─────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────┐
│         Session Keys (per circuit)          │
│  ┌─────────────────────────────────────┐   │
│  │ Derived via HKDF(shared_secret)     │   │
│  │ - Encryption key                    │   │
│  │ - MAC key                           │   │
│  │ - Nonce                             │   │
│  └─────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

### 3.3 Nonce Generation

**Current Implementation**: Random 12-byte nonces

```rust
// ✅ Secure: Random nonce per message
let mut nonce = [0u8; 12];
OsRng.fill_bytes(&mut nonce);
```

**Important**: Nonces MUST be unique per message. Reusing nonces compromises confidentiality.

---

## 4. Attack Surface

### 4.1 Network Layer

| Attack | Risk | Mitigation |
|--------|------|------------|
| Traffic analysis | High | Chaff, padding |
| Route correlation | High | Multiple paths |
| Timing attacks | Medium | Constant-time ops |
| DoS | High | Rate limiting |

### 4.2 Node Layer

| Attack | Risk | Mitigation |
|--------|------|------------|
| Node compromise | High | Layered encryption |
| Key theft | Critical | Hardware security |
| Malicious node | Medium | Reputation system |

### 4.3 Implementation

| Attack | Risk | Mitigation |
|--------|------|------------|
| Buffer overflow | Critical | Memory safety (Rust) |
| Side channels | Medium | Constant-time crypto |
| Randomness failure | Critical | OS CSPRNG |

---

## 5. Mitigations

### 5.1 Traffic Analysis

```rust
// Chaff traffic injection
if config.enable_chaff && randomaff_probability {
() < config.ch    // Inject decoy packets
    send_noise_packet();
}

// Padding
let padded = apply_uniform_padding(data, 512);
```

### 5.2 Route Correlation

- **Multi-path routing**: Traffic distributed across paths
- **Random selection**: Weighted random node selection
- **Circuit rotation**: Regular path changes

### 5.3 DoS Protection

```rust
// Rate limiting per node
let limiter = RateLimiter::new(100); // 100 req/s
if !limiter.check(&node_id) {
    return Err(Error::RateLimited);
}

// PoW for connection (optional)
if config.enable_pow {
    pow::verify(header, difficulty)?;
}
```

---

## 6. Known Limitations

### 6.1 Cryptographic Limitations

1. **Static Keys**: Nodes use long-term static keys
   - Risk: Compromise reveals past traffic if recorded
   - Mitigation: Use ephemeral session keys

2. **Limited Forward Secrecy**: 
   - Current implementation doesn't fully implement
   - Future: Implement DH ratcheting

3. **No Plausible Deniability**:
   - Digital signatures provide non-repudiation
   - May not be desired in all scenarios

### 6.2 Network Limitations

1. **Timing Attacks**:
   - Packet timing can correlate entry/exit
   - Mitigation: Add random delays (not implemented)

2. **Bandwidth Analysis**:
   - Traffic volume patterns visible
   - Mitigation: Constant-rate transmission (not implemented)

3. **Entry/Exit Exposure**:
   - First/last hops know identities
   - Mitigation: Use bridge nodes (not implemented)

### 6.3 Implementation Limitations

1. **No Perfect Clock Synchronization**:
   - Could improve timing mitigations

2. **Limited Key Rotation**:
   - Session keys persist too long

3. **No Pluggable Transports**:
   - Limited circumvention capability

---

## 7. Recommendations

### 7.1 For Research/Development

1. **Use in testing**: ✅ Safe for testing concepts
2. **Use in simulation**: ✅ Good for network simulation
3. **Contribute improvements**: ✅ Welcome
4. **Learn from code**: ✅ Educational value

### 7.2 For Production

**DO NOT USE** without:

1. ⚠️ Professional security audit
2. ⚠️ Cryptographic review
3. ⚠️ Formal verification
4. ⚠️ Penetration testing
5. ⚠️ Legal review

### 7.3 Alternative Solutions

For production anonymity needs, consider:

| Project | Type | Trust Model |
|---------|------|-------------|
| **Tor** | Onion routing | Mature, audited |
| **I2P** | Garlic routing | Distributed |
| **Nym** | Mixnet | Token-based |
| **Loopix** | Poisson mix | Academic |

---

## 8. Security Checklist

Before deployment, ensure:

- [ ] Random number generator is properly seeded
- [ ] Keys are stored securely (not in plaintext)
- [ ] Rate limiting is configured appropriately
- [ ] Logging doesn't reveal sensitive data
- [ ] Configuration is reviewed
- [ ] Network is isolated from production systems
- [ ] Regular security updates
- [ ] Incident response plan

---

## 9. Reporting Issues

If you discover security vulnerabilities:

1. **DO NOT** open a public issue
2. **DO NOT** share publicly
3. Contact maintainers privately
4. Allow time for remediation
5. Follow responsible disclosure

---

## References

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Crypto101](https://crypto101.io/)
- [Security Engineering](https://www.cl.cam.ac.uk/~rja14/book.html)
- [NIST Cryptographic Standards](https://csrc.nist.gov/projects/cryptographic-standards-and-guidelines)

---

*Document Version: 1.0*  
*Last Updated: 2025*
*This is a research document. Not for production use.*

