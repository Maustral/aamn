## Turnel / AAMN – Security & Anonymity Roadmap

Este documento define los pasos necesarios para acercar Turnel al nivel de anonimato y robustez de redes como Tor / I2P, partiendo del estado actual endurecido para producción privada.

> **Importante**: Este roadmap es ambicioso y de largo plazo.  
> No pretende decir que el proyecto ya ofrece anonimato al nivel de Tor/I2P, sino documentar qué sería necesario implementar y auditar.

---

### Fase 1 – Endurecimiento de protocolo dentro del diseño actual ✅

**Objetivo**: Maximizar la seguridad y el anonimato posible sin cambiar por completo la arquitectura.

- **Circuitos y rotación**
  - ✅ `Circuit` y `CircuitManager` definidos en `circuit.rs`, con expiración por tiempo (10 min) y limpieza periódica.
  - ✅ Selección de rutas multi‑hop mediante `PathFinder` en `routing.rs`.
  - ⚠️ Pendiente menor: contabilizar volumen por circuito y activar rotación también por umbral de bytes.

- **Forward secrecy y gestión de claves**
  - ✅ `HandshakeManager` implementa derivación de claves de sesión y rotación (`KEY_ROTATION_INTERVAL`), sin PSK por defecto en producción.
  - ⚠️ Pendiente menor: documentar con más detalle el ciclo de vida de claves en `docs/SECURITY.md`.

- **Cover traffic y padding**
  - ✅ `AAMNPacket::apply_padding` garantiza tamaños uniformes hasta `MAX_PACKET_SIZE`.
  - ✅ `padding.rs` implementa celdas fijas de 512 bytes (`Cell`) y gestores `TrafficPadding`, `CoverTrafficManager`, `TrafficShaper`.
  - ✅ `main.rs` integra `CoverTrafficManager` controlado por `AAMN_COVER_CELLS_PER_SEC`, generando tráfico de relleno onion incluso sin tráfico real.

- **Mitigaciones de logging y metadatos**
  - ✅ Logs de destinos SOCKS5 reducidos a nivel `debug` (no aparecen en `info`).
  - ✅ No se registran PSKs ni tokens en logs.
  - ⚠️ Pendiente menor: afinar ejemplos/recomendaciones de niveles de log en `docs/CONFIG.md`.

---

### Fase 2 – Modelo de red tipo Tor (guards, roles y selección de rutas) ⚙️ en progreso

**Objetivo**: Reducir la probabilidad de correlación entrada/salida por parte de un atacante que controla muchos nodos.

- **Entry Guards**
  - ✅ `NodeProfile` incluye `is_guard` y `RoutingTable` mantiene `guard_nodes`.
  - ✅ `PathFinder` prefiere guards como primer salto cuando están disponibles.
  - ⚠️ Pendiente: persistir el conjunto de guards preferidos y definir políticas de rotación muy lenta (requiere red real de relays).

- **Roles de nodo (entry / middle / exit)**
  - ✅ `NodeProfile` ampliado con `can_enter`, `can_middle`, `can_exit` (con defaults seguros).
  - ✅ `PathFinder::find_probabilistic_path`:
    - Primer hop: nodos `can_enter` (preferentemente guards).
    - Intermedios: nodos `can_middle`.
    - Último hop: nodos `can_exit`.
  - ⚠️ Pendiente: integrar información de país/AS cuando exista infraestructura que la aporte, para exigir diversidad geográfica.

- **Selección de rutas más robusta**
  - ✅ Selección ponderada (reputación, stake, latencia, ancho de banda) ya implementada.
  - ⚠️ Pendiente: restricciones adicionales (prefijos IP / AS) y documentación detallada en `docs/PROTOCOL.md` una vez haya datos de red reales.

---

### Fase 3 – Consenso / directorio y defensa contra Sybil (requiere infraestructura de red)

**Objetivo**: Evitar que un atacante pueble la red de nodos maliciosos y distorsione la vista de la red (Sybil).

- **Mecanismo de directorio o consenso**
  - 🧩 Pendiente sobre infraestructura física:
    - Directorios firmados (autoridades distribuidas) o consenso ligero sobre una DHT.
    - Listado verificable de nodos y sus claves públicas/roles.
  - `RoutingTable` y `NodeProfile` ya están listos para almacenar esa información cuando se defina el mecanismo.

- **Mitigaciones Sybil**
  - 🧩 Diseñar políticas (PoW, stake mínimo, límites a nuevos nodos) y desplegarlas a nivel de red y operación.
  - El código de `routing.rs`/`dht.rs` podrá aplicar esas reglas una vez se definan y exista una red real de múltiples operadores.

---

### Fase 4 – Anti‑censura y pluggable transports (requiere despliegue avanzado)

**Objetivo**: Resistir bloqueos y censura activa tipo DPI a nivel estatal.

- **Pluggable transports**
  - 🧩 Pendiente de diseño:
    - Definir una interfaz de transporte pluggable sobre `transport.rs`.
    - Implementar transports alternativos (QUIC camuflado, WebSockets/WebRTC, etc.) y probarlos en una red distribuida real.

- **Evasión de DPI**
  - 🧩 Pendiente:
    - Diseñar variaciones de patrones de longitud/tiempo y técnicas de camuflaje (SNI, dominios señuelo).
    - Requiere pruebas frente a DPI reales y evaluación legal en las jurisdicciones de despliegue.

---

### Fase 5 – Auditorías externas, verificación y gobernanza (trabajo externo al código)

**Objetivo**: Dar garantías comparables a proyectos como Tor/Nym más allá de la propia implementación.

- **Especificación formal actualizada**
  - ⚠️ Una vez estabilizadas las fases anteriores en una red real, actualizar `docs/PROTOCOL.md` para reflejar exactamente:
    - Circuitos, guards, selección de rutas y roles.
    - Gestión de claves y rotación observada en producción.
    - Mecanismos de consenso / directorio adoptados.

- **Auditorías externas**
  - 🧩 Trabajo de terceros:
    - Auditoría criptográfica independiente del diseño.
    - Auditoría de implementación Rust (handshake, routing, transport, padding, cover traffic).
    - Pentest sobre despliegues reales en la infraestructura física donde se ejecute Turnel.

- **Proceso de seguridad continuo**
  - ⚠️ `docs/SECURITY.md` ya esboza divulgación responsable; falta:
    - Definir cadencia de releases de seguridad/parches.
    - Establecer un proceso formal de respuesta a incidentes en la organización que opere la red.

---

### Estado actual (Turnel) frente al roadmap

Resumen rápido de lo que ya existe en el fork Turnel y lo que queda por hacer:

- ✅ Criptografía moderna (ChaCha20-Poly1305, X25519, Ed25519, HKDF, BLAKE2).
- ✅ Onion routing multi‑hop y DHT Kademlia básicos.
- ✅ Padding a celdas de 512 bytes y algún chaff.
- ✅ Plano de control endurecido (token + loopback) y manejo de PSK sin valores por defecto.
- ✅ Documentación de despliegue (`DEPLOYMENT.md`) y auditoría interna (`PRODUCTION_AUDIT.md`).
- ⚠️ Pendiente:
  - Diseño completo de guards y roles de nodo.
  - Selección de rutas avanzada y rotación estructurada de circuitos.
  - Mecanismos sólidos anti‑Sybil y posible consenso de red.
  - Pluggable transports y estrategias anti‑censura.
  - Auditorías externas formales de seguridad y cripto.

Este documento debe verse como guía de largo plazo para la evolución de Turnel
si el objetivo es acercarse al modelo de anonimato de Tor/I2P frente a atacantes estatales.

