# Guía de Configuración - AAMN

## Archivo de Configuración

AAMN usa un archivo de configuración TOML. Por defecto, busca `config.toml` en el directorio actual.

### Estructura del Archivo

```toml
# Configuración del nodo
[node]
# Identidad única del nodo (auto-generado si no existe)
node_id = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"

# Configuración de red
[network]
# Dirección y puerto de escucha
listen_addr = "0.0.0.0:9000"
# Dirección externa (para NAT traversal)
external_addr = "your-public-ip:9000"
# Nodos bootstrap para unirse a la red
bootstrap_nodes = [
    "bootstrap1.aamn.net:9000",
    "bootstrap2.aamn.net:9000",
]

# Configuración de seguridad
[security]
# Habilitar cifrado onion (recomendado)
onion_routing = true
# Número de saltos en la ruta (3-7)
circuit_hops = 3
# Timeout de circuito en segundos
circuit_timeout = 600
# HabilitarRate limiting
rate_limiting = true
# Solicitudes permitidas por segundo
rate_limit_rps = 100

# Configuración de privacidad
[privacy]
# Nivel de anonimato (1-10, 10 es máximo)
anonymity_level = 7
# Tiempo mínimo de retención en memoria
min_retention_sec = 300

# Configuración de métricas
[metrics]
# Habilitar métricas
enabled = true
# Puerto del endpoint de métricas
prometheus_port = 9090
# Health check endpoint
health_check_port = 8080

# Configuración de logging
[logging]
# Nivel de log: trace, debug, info, warn, error
level = "info"
# Formato: text, json
format = "text"
# Archivo de log
log_file = "aamn.log"
# Habilitar rotación de logs
rotation = true
# Tamaño máximo del archivo de log (MB)
max_file_size = 10
# Número máximo de archivos de log
max_files = 5

# Configuración de persistencia
[storage]
# Directorio de datos
data_dir = "./data"
# Intervalo de guardado (segundos)
save_interval = 300
```

## Variables de Entorno

También puedes usar variables de entorno:

```bash
# Configuración de red
export AAMN_LISTEN_ADDR="0.0.0.0:9000"
export AAMN_BOOTSTRAP_NODES="bootstrap1.aamn.net:9000"

# Seguridad
export AAMN_CIRCUIT_HOPS=3

# Logging
export AAMN_LOG_LEVEL="debug"
```

## Configuración Avanzada

### Alta Disponibilidad

```toml
[ha]
# Habilitar modo HA
enabled = true
# Nodos pares para redundancia
peer_nodes = [
    "node2.aamn.net:9000",
]
# Intervalo de health check (segundos)
health_check_interval = 30
```

### Configuración de Red Avanzada

```toml
[network.advanced]
# Tamaño máximo de paquete
max_packet_size = 1450
# MTU de la interfaz TUN
tun_mtu = 1500
# Habilitar NAT traversal
nat_traversal = true
# STUN servers
stun_servers = [
    "stun.l.google.com:19302",
]
# Tiempo de espera de conexión (segundos)
connection_timeout = 30
```

### Configuración de Circuitos

```toml
[circuit]
# Longitud mínima de circuito
min_hops = 3
# Longitud máxima de circuito  
max_hops = 7
# Rotar circuitos automáticamente
auto_rotate = true
# Umbral de tráfico para rotación (bytes)
rotate_threshold = 1048576  # 1MB
```

### Configuración de Criptografía

```toml
[crypto]
# Algoritmo de cifrado: chacha20poly1305, aes256gcm
cipher = "chacha20poly1305"
# Longitud de clave (bits)
key_size = 256
# Verificar certificados
verify_certificates = true
```

## Ejemplos de Configuración

### Nodo de Usuario Final

```toml
[node]
node_id = "auto"

[network]
listen_addr = "127.0.0.1:9000"

[security]
onion_routing = true
circuit_hops = 3

[privacy]
anonymity_level = 7

[logging]
level = "info"
```

### Nodo Relay

```toml
[node]
# Generado automáticamente

[network]
listen_addr = "0.0.0.0:9000"
external_addr = "your-public-ip:9000"
bootstrap_nodes = ["bootstrap.aamn.net:9000"]

[security]
onion_routing = true
circuit_hops = 3

[metrics]
enabled = true
prometheus_port = 9090

[logging]
level = "debug"
```

### Nodo de Alta Velocidad

```toml
[network]
listen_addr = "0.0.0.0:9000"
bootstrap_nodes = ["bootstrap.aamn.net:9000"]

[security]
circuit_hops = 5
rate_limit_rps = 1000

[circuit]
max_hops = 7
auto_rotate = true
rotate_threshold = 10485760  # 10MB

[resources]
max_bandwidth_mbps = 1000
```

## Validación de Configuración

```bash
# Validar archivo de configuración
aamn validate-config --config config.toml
```

## Configuración en Producción

Para nodos en producción, considera:

1. **Seguridad**: Usa certificados firmados
2. **Logging**: Configura rotación de logs
3. **Métricas**: Habilita Prometheus endpoint
4. **Red**: Configura NAT traversal
5. **Recursos**: Limita ancho de banda

```toml
[security]
verify_certificates = true
onion_routing = true
circuit_hops = 5

[logging]
level = "warn"
rotation = true
max_files = 10

[metrics]
enabled = true
prometheus_port = 9090

[resources]
max_connections = 1000
max_bandwidth_mbps = 100
