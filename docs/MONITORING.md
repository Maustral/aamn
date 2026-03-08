# 📊 Guía de Monitoreo - AAMN Network

## 1. Prometheus + Grafana Setup

### Configuración Prometheus

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'aamn-node'
    static_configs:
      - targets: ['localhost:9000']
    metrics_path: '/metrics'
```

### Métricas Disponibles

| Métrica | Tipo | Descripción |
|---------|------|-------------|
| `aamn_connections_total` | Counter | Conexiones totales |
| `aamn_packets_processed` | Counter | Paquetes procesados |
| `aamn_dht_nodes` | Gauge | Nodos DHT activos |
| `aamn_circuit_builds` | Counter | Circuitos creados |
| `aamn_rate_limit_exceeded` | Counter | Rate limits superados |
| `aamn_onion_layers` | Gauge | Capas de cifrado |
| `aamn_latency_ms` | Histogram | Latencia de red |
| `aamn_bandwidth_bytes` | Gauge | Ancho de banda |

---

## 2. Grafana Dashboard JSON

```json
{
  "dashboard": {
    "title": "AAMN Network Monitor",
    "panels": [
      {
        "title": "Conexiones Activas",
        "targets": [
          {"expr": "aamn_connections_total"}
        ],
        "type": "graph"
      },
      {
        "title": "Paquetes/seg",
        "targets": [
          {"expr": "rate(aamn_packets_processed[5m])"}
        ],
        "type": "graph"
      },
      {
        "title": "Nodos DHT",
        "targets": [
          {"expr": "aamn_dht_nodes"}
        ],
        "type": "gauge"
      },
      {
        "title": "Latencia P50",
        "targets": [
          {"expr": "histogram_quantile(0.50, aamn_latency_ms)"}
        ],
        "type": "graph"
      },
      {
        "title": "Rate Limits Excedidos",
        "targets": [
          {"expr": "rate(aamn_rate_limit_exceeded[5m])"}
        ],
        "type": "graph",
        "alert": {
          "enabled": true,
          "threshold": 10
        }
      }
    ]
  }
}
```

---

## 3. Alertas Recommended

### Alertas Críticas

| Alerta | Condición | Acción |
|--------|-----------|--------|
| DDoS Activo | `rate(aamn_rate_limit_exceeded[1m]) > 100` | Notificar inmediatamente |
| Nodos Caídos | `aamn_dht_nodes < 3` | Revisar red |
| Latencia Alta | `histogram_quantile(0.95, aamn_latency_ms) > 500` | Investigar |

### Alertas de Warning

| Alerta | Condición | Acción |
|--------|-----------|--------|
| Alto Rate Limiting | `rate(aamn_rate_limit_exceeded[5m]) > 10` | Monitorear |
| Bajo Ancho de Banda | `aamn_bandwidth_bytes < 1000` | Verificar conexión |

---

## 4. Runbooks

### runbook: alta_latencia.md

```
# Alta Latencia

## Síntomas
- Latencia P95 > 500ms

## Diagnóstico
1. Verificar estado de red: `netstat -s`
2. Revisar nodos DHT: `aamn_dht_nodes`
3. Check circuit build time

## Solución
1. Seleccionar nodos de menor latencia
2. Reducir número de saltos
3. Verificar conectividad
```

### runbook: ddos_detected.md

```
# DDoS Detectado

## Síntomas
- Rate limits > 100/min
- Conexiones simultáneas > 1000

## Contención
1. Habilitar modo estricto de rate limiting
2. Bloquear IPs suspectas
3. Notificar al equipo de seguridad

## Recuperación
1. Monitorear por 30 minutos
2. Gradualmente reducir restricciones
3. Documentar incidente
```

---

## 5. Logs Estructurados

### Formato JSON

```json
{
  "timestamp": "2025-01-01T00:00:00Z",
  "level": "INFO",
  "component": "handshake",
  "message": "Connection established",
  "peer_id": "abc123...",
  "latency_ms": 45
}
```

### Logs de Seguridad

| Evento | Level | Campos |
|--------|-------|--------|
| Conexión aceptada | INFO | peer_id, ip |
| Conexión rechazada | WARN | reason, ip |
| Rate limit excedido | WARN | peer_id, limit |
| Error de cifrado | ERROR | peer_id, error |
| Nodo sospechoso | ALERT | peer_id, reason |

---

## 6. Backup y Restore

### Comandos de Backup

```bash
# Backup de configuración
cp config.toml config.toml.backup

# Backup de claves (cifrado)
tar czf keys_backup.tar.gz -C ~/.aamn keys/

# Backup de estado DHT
curl localhost:9000/dht/export > dht_state.json
```

### Comandos de Restore

```bash
# Restore configuración
cp config.toml.backup config.toml

# Restore estado DHT
curl -X POST localhost:9000/dht/import -d @dht_state.json

# Reiniciar servicio
systemctl restart aamn
```

---

## 7. Health Checks

### Endpoint de Health

```bash
# Verificar salud del nodo
curl localhost:9000/health

# Respuesta esperada:
{
  "status": "healthy",
  "version": "1.0.0",
  "uptime_seconds": 3600,
  "connections": 42,
  "dht_nodes": 15
}
```

### Verificaciones Automáticas

| Check | Frecuencia | Acción |
|-------|------------|--------|
| /health | 30 seg | Alerta si fail |
| /metrics | 15 seg | Scraping |
| /dht/status | 1 min | Warn si nodos < 5 |

---

## 8. Notificaciones

### Configuración de Alerts

```yaml
# alerts.yml
groups:
  - name: aamn_alerts
    rules:
      - alert: HighLatency
        expr: histogram_quantile(0.95, aamn_latency_ms) > 500
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Alta latencia detectada"
          
      - alert: DDoS
        expr: rate(aamn_rate_limit_exceeded[1m]) > 100
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Posible DDoS en curso"
```

### Canales de Notificación

| Canal | Usar para |
|-------|-----------|
| Slack #alerts | Warnings |
| PagerDuty | Critical |
| Email | Resumen diario |

---

*Documento para operación 24/7*
*Versión: 1.0*
