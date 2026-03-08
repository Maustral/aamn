# 🔐 AAMN — Adaptive Anonymous Mesh Network

<div align="center">
  <img src="assets/aamn_hero.svg" alt="AAMN Animated Cover" width="100%">

  <br />

  [![Build Status](https://img.shields.io/badge/Build-Passing-brightgreen?style=for-the-badge&logo=github)](#)
  [![Rust](https://img.shields.io/badge/Rust-1.74+-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
  [![License](https://img.shields.io/badge/License-MIT%2FApache_2.0-blue?style=for-the-badge)](#licencia)
  [![Version](https://img.shields.io/badge/Version-0.2.0-purple?style=for-the-badge)](#)
  [![Security Audit](https://img.shields.io/badge/Security_Audit-100%25-brightgreen?style=for-the-badge)](#-seguridad)

  <p align="center">
    <strong>Enrutamiento en malla P2P seguro, anónimo y denegable.</strong><br>
    <i>Ningún nodo central. Sin IP's rastreables. Criptografía de grado militar.</i>
  </p>
</div>

---

## ⚡ Características Principales

| Característica | Descripción |
|---------------|-------------|
| 🔒 **Cifrado Onion** | ChaCha20-Poly1305 con múltiples capas de cifrado |
| 🌐 **Routing Anónimo** | Mínimo 3 saltos, cada nodo solo conoce el siguiente |
| 🔑 **Noise Protocol** | Handshake IKpsk2 con Perfect Forward Secrecy |
| 🛡️ **Protección de Tráfico** | Padding fijo de 1450 bytes, traffic shaping |
| ⚡ **Alto Rendimiento** | QUIC transport, DHT Kademlia distribuido |
| 🔄 **Auto-escalable** | Descubrimiento de nodos sin servidores centrales |

---

## 📋 Índice

1. [Instalación](#-instalación)
2. [Uso](#-uso)
3. [Configuración](#-configuración)
4. [Arquitectura](#-arquitectura)
5. [Seguridad](#-seguridad)
6. [API](#-api)
7. [Monitoreo](#-monitoreo)
8. [Contribución](#-contribución)
9. [Licencia](#-licencia)

---

## 🚀 Instalación

### Requisitos

- **Rust 1.70+** con soporte para `nightly`
- **Windows 10/11** o **Linux** (macOS en desarrollo)
- **Privilegios de Administrador** (para crear interfaz TUN)
- **wintun.dll** (solo Windows)

### Compilación

```bash
# Clonar el repositorio
git clone https://github.com/AAMN-Network/AAMN.git
cd AAMN

# Compilar en modo debug
cargo build

# Compilar en modo release (recomendado para producción)
cargo build --release
```

### Configuración Inicial

Ver [INSTALL.md](docs/INSTALL.md) para instrucciones detalladas.

---

## 📖 Uso

### Iniciar un Nodo

```bash
# Iniciar con configuración por defecto
cargo run --release --bin aamn

# Especificar puerto
cargo run --release -- --port 9000

# Usar archivo de configuración personalizado
cargo run --release -- --config config.toml
```

### Conectar a la Red

```bash
# Conectar a nodos bootstrap
cargo run --release -- --bootstrap bootstrap.aamn.network:9000

# Modo daemon en background
cargo run --release -- --daemon
```

Ver [USAGE.md](docs/USAGE.md) para más ejemplos.

---

## ⚙️ Configuración

Crear archivo `config.toml`:

```toml
[network]
listen_addr = "0.0.0.0:9000"
use_quic = true
max_connections = 100

[security]
onion_layers = 3
enable_hmac = true
enable_pow = true
pow_difficulty = 20

[performance]
max_packet_size = 1450
fragment_size = 512
rate_limit_rps = 100

[logging]
level = "info"
file_enabled = true
file_path = "aamn.log"
```

Ver [CONFIG.md](docs/CONFIG.md) para todas las opciones.

---

## 🏗️ Arquitectura

```
┌─────────────────────────────────────────────────────────────┐
│                      AAMN Network                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐ │
│  │ Client  │───▶│ Node A  │───▶│ Node B  │───▶│ Node C  │ │
│  │ (Entry) │    │ (Relay) │    │ (Relay) │    │(Exit)   │ │
│  └─────────┘    └─────────┘    └─────────┘    └─────────┘ │
│       │              │              │              │       │
│       ▼              ▼              ▼              ▼       │
│  ┌─────────────────────────────────────────────────────┐  │
│  │              ONION ENCRYPTION LAYERS                │  │
│  │  Layer 3: Public Key of Node C                      │  │
│  │  Layer 2: Public Key of Node B + Encrypted Payload  │  │
│  │  Layer 1: Public Key of Node A + Encrypted Payload  │  │
│  └─────────────────────────────────────────────────────┘  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Componentes Principales

| Módulo | Descripción |
|--------|-------------|
| `src/crypto.rs` | Cifrado onion, X25519, Ed25519, ChaCha20-Poly1305 |
| `src/handshake.rs` | Noise Protocol IKpsk2, forward secrecy |
| `src/dht.rs` | Kademlia DHT para descubrimiento de nodos |
| `src/circuit.rs` | Gestión de circuitos onion |
| `src/transport.rs` | QUIC transport con TLS 1.3 |
| `src/rate_limiter.rs` | Token Bucket + Sliding Window |
| `src/padding.rs` | Cell-based routing, traffic shaping |

Ver [PROTOCOL.md](docs/PROTOCOL.md) para detalles del protocolo.

---

## 🔒 Seguridad

### Criptografía

| Componente | Algoritmo | Notas |
|-----------|-----------|-------|
| Cifrado Simétrico | ChaCha20-Poly1305 | AEAD seguro |
| Intercambio de Claves | X25519 | Curve25519 DH |
| Firmas Digitales | Ed25519 | Identidades inmutables |
| Protocolo de Enlace | Noise IKpsk2 | Perfect Forward Secrecy |
| Hashing | BLAKE2b/SHA-256 | Integridad y HMAC |
| RNG | OsRng | Aleatoriedad segura |

### Auditoría de Seguridad

El proyecto ha sido auditado internamente. Ver [SECURITY_AUDIT.md](docs/SECURITY_AUDIT.md).

### Checklist de Seguridad

- ✅ Handshake Noise real implementado
- ✅ Forward secrecy habilitado
- ✅ Verificación de certificados TLS
- ✅ Nonce reuse prevention
- ✅ KDF estándar (HKDF)
- ✅ Padding de tráfico cell-based
- ✅ Rate limiting global
- ✅ Detección de DDoS
- ✅ TLS mutual authentication
- ✅ Logging de seguridad

---

## 📚 API

### Ejemplo: Crear un Mensaje Cifrado

```rust
use aamn::{OnionEncryptor, NodeIdentity};

// Crear identidad
let identity = NodeIdentity::generate();

// Cifrar mensaje para múltiples saltos
let keys = vec![key1, key2, key3];
let nodes = vec![node1_id, node2_id, node3_id];
let encrypted = OnionEncryptor::wrap(data, &keys, &nodes)?;
```

### Ejemplo: Iniciar Handshake

```rust
use aamn::HandshakeManager;

let psk = b"mi-psk-secreto";
let manager = HandshakeManager::new(&psk);

// Iniciar handshake como cliente
let output = manager.initiate_handshake(&peer_public_key)?;
let message = output.handshake_message;
```

Ver [API.md](docs/API.md) para la documentación completa.

---

## 📊 Monitoreo

### Métricas Prometheus

```bash
# Endpoint de métricas
http://localhost:9090/metrics
```

### Métricas Disponibles

| Métrica | Descripción |
|---------|-------------|
| `aamn_circuits_active` | Circuitos activos |
| `aaml_packets_routed` | Paquetes enrutados |
| `aamn_nodes_connected` | Nodos conectados |
| `aamn_latency_ms` | Latencia media |
| `aamn_bandwidth_kbps` | Ancho de banda |
| `aamn_errors_total` | Errores totales |

Ver [MONITORING.md](docs/MONITORING.md) para dashboards y alertas.

---

## 🤝 Contribución

### Requisitos

- Rust 1.70+
- Cargo
- git

### Configuración de Desarrollo

```bash
# Fork y clonar
git clone https://github.com/TU_USUARIO/AAMN.git
cd AAMN

# Crear rama feature
git checkout -b feature/mi-feature

# Ejecutar tests
cargo test

# Verificar estilo
cargo fmt --check
cargo clippy
```

### Estándares de Código

- Usar `cargo fmt` antes de commits
- No usar `unsafe` sin justificación
- Todos los tests deben pasar
- Documentar APIs públicas

---

## 📄 Licencia

Este proyecto está licenciado bajo **MIT License** - ver [LICENSE](LICENSE) para detalles.

### Dependencias y Licencias

| Paquete | Licencia |
|---------|----------|
| ring | Apache 2.0 / MIT / ISC |
| snow | MIT |
| x25519-dalek | MIT / Apache 2.0 |
| ed25519-dalek | MIT / Apache 2.0 |
| chacha20poly1305 | Apache 2.0 / MIT |
| tokio | MIT |
| quinn | MIT / Apache 2.0 |

---

## ⚠️ Aviso Legal

```
Este software se proporciona "tal cual" sin garantías de ningún tipo.

El uso de redes de anonimato puede ser ilegal en ciertas jurisdicciones.
El autor no se hace responsable del uso que se dé a este software.

El usuario es responsable de:
- Cumplir con las leyes locales
- Entender los riesgos de privacidad
- Verificar la implementación criptográfica

Se recomienda auditar el código antes de uso en producción.
Consulte con un profesional del derecho antes de operar nodos.
```

Ver [ASPECTOS_LEGALES.md](ASPECTOS_LEGALES.md) para información completa.

---

## 📞 Contacto

| Canal | Enlace |
|-------|--------|
| GitHub | [github.com/AAMN-Network/AAMN](https://github.com/AAMN-Network/AAMN) |
| Creador | [github.com/Maustral](https://github.com/Maustral) |
| Instagram | [@yojancelm02](https://instagram.com/yojancelm02) |
| Security | security@aamn.network |

---

<div align="center">

*Construido con 🔐 y 🦀*

**AAMN** - *Red de comunicaciones anónimas*

</div>

