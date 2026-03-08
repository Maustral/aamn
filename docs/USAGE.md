# Guía de Uso - AAMN

## Comandos Básicos

### Iniciar el Nodo

```bash
# Iniciar con configuración por defecto
aamn start

# Especificar puerto
aamn start --port 9000

# Usar nodo bootstrap
aamn start --bootstrap bootstrap.aamn.net:9000

# Iniciar como daemon
aamn start --daemon

# Con configuración personalizada
aamn start --config config.toml
```

### Detener el Nodo

```bash
aamn stop
```

### Estado del Nodo

```bash
aamn status
```

Salida esperada:
```
=== AAMN Node Status ===
Version: 0.2.0
Status: Running
Port: 9000
Peers: 5
 Circuits: 3
Uptime: 3600s
=======================
```

## Gestión de Conexiones

### Conectar a un Peer

```bash
aamn connect 192.168.1.100:9000
```

### Listar Peers Conectados

```bash
aamn peers
```

### Ver Información de un Peer

```bash
aamn peer-info <peer_id>
```

## Gestión de Identidades

### Generar Nueva Identidad

```bash
aamn gen-identity --output identity.json
```

### Ver Identidad Actual

```bash
aamn show-identity
```

### Importar Identidad

```bash
aamn import-identity --from identity.json
```

## Gestión de Circuitos

### Ver Circuitos Activos

```bash
aamn circuits
```

### Crear Nuevo Circuito

```bash
aamn circuit create --hops 5
```

### Destruir Circuito

```bash
aamn circuit destroy <circuit_id>
```

## Métricas y Monitoreo

### Ver Métricas en Consola

```bash
aamn metrics
```

### Endpoint Prometheus

Las métricas están disponibles en:
```
http://localhost:9090/metrics
```

### Health Check

```
http://localhost:8080/health
```

Respuesta:
```json
{
  "status": "healthy",
  "uptime_secs": 3600,
  "version": "0.2.0"
}
```

## Logging

### Ver Logs en Tiempo Real

```bash
# Todos los logs
aamn logs

# Solo errores
aamn logs --level error

# Filtrar por componente
aamn logs --component network
```

### Archivos de Log

Los logs se almacenan en:
- Linux/macOS: `~/.local/share/aamn/aamn.log`
- Windows: `%LOCALAPPDATA%\aamn\aamn.log`

## Configuración

### Validar Configuración

```bash
aamn validate-config --config config.toml
```

### Ver Configuración Actual

```bash
aamn config show
```

### Actualizar Configuración en Vivo

```bash
# Actualizar nivel de log
aamn config set logging.level debug

# Actualizar rate limiting
aamn config set security.rate_limit_rps 200
```

## Modo Daemon

### Iniciar como Servicio (Linux)

```bash
# Instalar servicio
sudo aamn install-service

# Iniciar
sudo aamn start

# Detener
sudo aamn stop

# Ver estado
sudo aamn status
```

### Iniciar como Servicio (Windows)

```bash
# Instalar servicio
aamn install-service

# Iniciar
aamn start

# Detener
aamn stop
```

## Casos de Uso

### Uso como Cliente VPN

```bash
# Iniciar nodo
aamn start --config client.toml

# Configurar tu aplicación para usar localhost:9000 como proxy
```

### Ejecutar un Nodo Relay

```bash
# Configurar como relay
aamn start --relay --public-ip $(curl ifconfig.me)
```

### Testing Local

```bash
# Iniciar nodos de prueba
aamn start --port 9000 &
aamn start --port 9001 &
aamn start --port 9002 &

# Conectar nodos
aamn connect localhost:9001
aamn connect localhost:9002
```

## Solución de Problemas

### Nodo No Inicia

```bash
# Ver errores detallados
aamn start -vvv

# Verificar configuración
aamn validate-config
```

### No Conecta a la Red

```bash
# Verificar conectividad
ping bootstrap.aamn.net

# Probar con otro bootstrap
aamn start --bootstrap backup.aamn.net:9000
```

### Rendimiento Lento

```bash
# Ver métricas
aamn metrics

# Reducir saltos
aamn config set security.circuit_hops 3

# Ver circuitos
aamn circuits
```

### Alto Uso de Memoria

```bash
# Reducir retention
aamn config set privacy.min_retention_sec 60

# Limitar conexiones
aamn config set resources.max_connections 100
```

## Scripting

### Ejemplo: Conectar a Múltiples Peers

```bash
#!/bin/bash
PEERS=("10.0.0.1:9000" "10.0.0.2:9000" "10.0.0.3:9000")

for peer in "${PEERS[@]}"; do
    echo "Conectando a $peer..."
    aamn connect "$peer"
done
```

### Ejemplo: Monitoreo

```bash
#!/bin/bash
while true; do
    clear
    echo "=== Métricas ==="
    aamn metrics
    sleep 5
done
```

### Ejemplo: Reinicio Automático

```bash
#!/bin/bash
while true; do
    aamn start
    echo "Nodo detenido, reiniciando en 5s..."
    sleep 5
done
```

## API REST

AAMN proporciona una API REST (opcional):

```bash
# Obtener estado
curl http://localhost:8080/api/v1/status

# Obtener métricas
curl http://localhost:8080/api/v1/metrics

# Obtener peers
curl http://localhost:8080/api/v1/peers

# Crear circuito
curl -X POST http://localhost:8080/api/v1/circuit
```

## Variables de Entorno Útiles

```bash
# Nivel de debug
export RUST_LOG=debug

# Puerto
export AAMN_PORT=9000

# Archivo de configuración
export AAMN_CONFIG=/ruta/config.toml
```

## Atajos de Teclado (modo interactivo)

- `Ctrl+C`: Detener nodo gracefully
- `Ctrl+L`: Limpiar pantalla
- `Ctrl+R`: Recargar configuración
- `q`: Salir
