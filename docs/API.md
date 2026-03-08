# AAMN API Documentation

## Módulos Principales

### Crypto (`src/crypto.rs`)

#### `NodeIdentity`
Identidad криптográfica de un nodo.

```rust
// Generar nueva identidad
let identity = NodeIdentity::generate();

// Obtener ID público del nodo
let node_id = identity.public_id(); // [u8; 32]

// Derivar clave compartida con otro nodo (Diffie-Hellman)
let shared_secret = identity.derive_shared_secret(&their_public_key);
```

#### `OnionEncryptor`
Cifrado multicapa tipo onion.

```rust
// Cifrar payload con múltiples capas
let wrapped = OnionEncryptor::wrap(
    payload,           // Datos a cifrar
    &keys,             // Claves para cada salto [[u8; 32]]
    &next_hops,        // NodeIDs de siguiente salto [[u8; 32]]
)?;

// Descifrar una capa
let (next_node, inner_payload) = OnionEncryptor::unwrap(
    wrapped_data,      // Datos cifrados
    &shared_key,      // Clave compartida [u8; 32]
)?;
```

---

### Routing (`src/routing.rs`)

#### `RoutingTable`
Tabla de enrutamiento de nodos.

```rust
let mut table = RoutingTable::new();

// Actualizar información de un nodo
table.update_node(NodeProfile {
    id: [u8; 32],
    endpoint: String,
    last_seen: DateTime<Utc>,
    latency_ms: u32,
    bandwidth_kbps: u32,
    reputation: f32,
    staked_amount: u64,
});

// Buscar nodos
let nodes = table.find_nodes(&[u8; 32], 5); // Por ID específico

// Persistencia
table.save_to_disk("routing.json")?;
let table = RoutingTable::load_from_disk("routing.json")?;
```

#### `PathFinder`
Selección de rutas probabilística.

```rust
let finder = PathFinder::new(routing_table);

// Encontrar ruta con número específico de saltos
let path = finder.find_probabilistic_path(3)?;

// Encontrar ruta a un destino específico
let path = finder.find_path_to_destination(&dest_id, 5)?;
```

---

### Fragment (`src/fragment.rs`)

#### `FragmentationManager`
Gestión de fragmentación y reconstrucción.

```rust
let manager = FragmentationManager::new();

// Fragmentar datos
let fragments = manager.fragment(&data); 
// Retorna: Vec<(fragment_id, data, is_last)>

// Reconstruir datos
let reconstructed = manager.reconstruct(fragments)?;

// Firmar fragmento (autenticidad)
let (hmac, signed_data) = manager.sign_fragment(&data, &key)?;

// Verificar fragmento
let verified_data = manager.verify_fragment(&signed_data, &key)?;
```

---

### Handshake (`src/handshake.rs`)

#### `HandshakeManager`
Protocolo Noise para establecimiento de sesión.

```rust
let manager = HandshakeManager::new();

// Iniciar handshake como cliente
let output = manager.initiate_handshake(&their_public_key)?;

// Responder a handshake como servidor
let response = manager.respond_to_handshake(&handshake_message)?;

// Completar handshake
let success = manager.complete_handshake(&response)?;

// Cifrar mensajes
let encrypted = manager.encrypt(&plaintext)?;

// Descifrar mensajes
let decrypted = manager.decrypt(&ciphertext)?;
```

---

### Rate Limiter (`src/rate_limiter.rs`)

#### `RateLimiter`
Protección contra ataques DoS.

```rust
// Crear rate limiter (solicitudes por segundo)
let limiter = RateLimiter::new(100);

// Verificar si se permite solicitud
if limiter.check(&node_id) {
    // Permitido
} else {
    // Rate limit excedido
}

// Obtener información del bucket
let info = limiter.get_bucket_info(&node_id);
```

#### `SlidingWindowRateLimiter`
Rate limiting con ventana deslizante.

```rust
let limiter = SlidingWindowRateLimiter::new(100, 60); // 100 reqs por 60 segundos

if limiter.check(&node_id) {
    // Permitido
}
```

---

### Metrics (`src/metrics.rs`)

#### `NetworkMetrics`
Métricas globales de red.

```rust
let metrics = NetworkMetrics::new();

// Registrar eventos
metrics.inc_packets_sent(1);
metrics.add_bytes_encrypted(1024);
metrics.inc_active_circuits();
metrics.dec_active_circuits();

// Obtener resumen
let summary = metrics.summary();
println!("{}", summary);
```

#### `TrafficMetricsCollector`
Métricas por nodo.

```rust
let collector = TrafficMetricsCollector::new();

// Registrar tráfico
collector.record_send(&node_id, 1024, 50); // bytes, latency_ms
collector.record_receive(&node_id, 512);

// Obtener métricas de un nodo
let node_metrics = collector.get_metrics(&node_id);
```

---

### Config (`src/config.rs`)

#### `Config`
Configuración centralizada del nodo.

```rust
// Cargar desde archivo
let config = Config::load_from_file("config.json")?;

// Cargar desde variables de entorno
let config = Config::load_from_env();

// Usar valores por defecto
let config = Config::default();

// Validar configuración
config.validate()?;

// Guardar configuración
config.save_to_file("config.json")?;

// Usar ConfigBuilder
let config = ConfigBuilder::new()
    .with_network(NetworkConfig { 
        listen_addr: "0.0.0.0:9000".parse()?,
        ..Default::default() 
    })
    .build()?;
```

---

### Transport (`src/transport.rs`)

#### `TransportLayer`
Capa de transporte QUIC.

```rust
let transport = TransportLayer::new("0.0.0.0:9000".parse()?)?;

// Conectar a un nodo
let conn = transport.connect("10.0.0.1:9000".parse()?).await?;

// Enviar paquete
transport.send_packet(&conn, data).await?;

// Recibir paquetes
while let Some(packet) = transport.listen_packets(&conn).await {
    // Procesar paquete
}
```

---

## Estructuras de Datos

### `AAMNPacket`
Paquete del protocolo AAMN.

```rust
let packet = AAMNPacket::new(payload, fragment_id);
let padded = packet.apply_padding();
let unpadded = packet.remove_padding();
```

### `Circuit`
Circuito de comunicación.

```rust
let circuit = Circuit::new(entry_node, exit_node);

// Verificar si necesita rotación
if circuit.should_rotate(1_000_000, bytes_sent) {
    // Rotar circuito
}
```

### `NodeProfile`
Perfil de un nodo en la red.

```rust
let profile = NodeProfile {
    id: [u8; 32],
    endpoint: "10.0.0.1:9000".to_string(),
    last_seen: Utc::now(),
    latency_ms: 50,
    bandwidth_kbps: 10000,
    reputation: 0.95,
    staked_amount: 1000,
};
```

---

## Uso desde main.rs

```rust
use aamn::{
    crypto::NodeIdentity,
    network::SecurityEngine,
    routing::{RoutingTable, NodeProfile},
    config::Config,
    metrics::NetworkMetrics,
};

fn main() -> Result<()> {
    // Cargar configuración
    let config = Config::load_from_file("config.json")
        .unwrap_or_default();
    
    // Inicializar tabla de routing
    let mut routing_table = RoutingTable::new();
    
    // ... agregar nodos ...
    
    // Crear motor de seguridad
    let engine = SecurityEngine::new(routing_table);
    
    // Inicializar métricas
    let metrics = NetworkMetrics::new();
    
    // Procesar tráfico
    let packet = engine.protect_traffic_auto(data, 3)?;
    metrics.inc_packets_sent(1);
    
    Ok(())
}
```

