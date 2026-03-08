## AAMN / Turnel – Guía de Despliegue en Producción

Este documento resume cómo desplegar el nodo AAMN y el dashboard en un entorno real, endureciendo la seguridad para uso con usuarios reales y dejando el repositorio listo para GitHub sin secretos.

---

### 1. Componentes

- **Nodo AAMN (backend Rust)**  
  - Provee la red P2P, cifrado onion, SOCKS5 local y APIs de control (gRPC + REST).
- **Dashboard Web (frontend React/Vite)**  
  - Interfaz de monitorización y control que habla con las APIs REST/gRPC del nodo.

En producción se recomienda ejecutarlos con `docker-compose`.

---

### 2. Variables de Entorno Críticas

- **Plano de control (APIs de administración)**  
  - `AAMN_CONTROL_TOKEN` (en el contenedor `aamn-node`):
    - Token compartido que protege las APIs gRPC (`50051`) y REST (`50052`).
    - El dashboard lo envía como `Authorization: Bearer <token>`.
    - Debe ser un valor largo y aleatorio (mínimo 32 caracteres).
- **Clave precompartida de red (PSK)**  
  - Opciones (elegir solo una):
    - `AAMN_PSK`: PSK en texto (solo para entornos controlados).
    - `AAMN_PSK_FILE`: ruta a un archivo dentro del contenedor con la PSK.

> **Importante**: No subas nunca PSKs reales ni tokens reales a GitHub.  
> Usa `.env`, secretos de CI/CD o variables de entorno en el servidor.

---

### 3. Despliegue con Docker Compose

El archivo `docker-compose.yml` ya viene endurecido para producción:

- Puerto P2P `9000` expuesto públicamente (para la red).
- Puerto SOCKS5 `1080` ligado a loopback: `127.0.0.1:1080:1080` (solo local).
- APIs de control:
  - gRPC `50051`: `127.0.0.1:50051:50051` (solo local).
  - REST `50052`: `127.0.0.1:50052:50052` (solo local).
- Dashboard HTTP expuesto en `80:80`.

Ejemplo de despliegue:

```bash
docker compose pull   # si ya tienes imágenes
docker compose up -d  # arranca nodo + dashboard
```

Antes de levantarlo en producción, exporta un token fuerte:

```bash
export AAMN_CONTROL_TOKEN="cambia-esto-por-un-token-fuerte"
export AAMN_PSK_FILE="/ruta/secreta/psk.txt"   # opcional, recomendado
```

Y ajusta `docker-compose.yml` o usa un `docker-compose.override.yml` para inyectar esos valores sin commitearlos.

---

### 4. Acceso Seguro al Dashboard

1. Abre en el navegador: `http://<tu-servidor>` (o `http://localhost`).
2. Verás la pantalla **“AAMN Dashboard Login”**.
3. Introduce el mismo token configurado en `AAMN_CONTROL_TOKEN`.
4. A partir de ahí:
   - El dashboard usará el header `Authorization: Bearer <token>` en:
     - `http://localhost:50052/api/status`
     - `http://localhost:50052/api/peers`
     - `http://localhost:50052/api/noise`
     - `http://localhost:50052/api/stop`
   - El cliente gRPC-Web también adjunta ese token.

> **Recomendado**: Exponer el dashboard solo detrás de HTTPS (reverse proxy tipo Nginx/Caddy) si se accede desde otras máquinas de la red.

---

### 5. Checklist de Seguridad para Producción

- **Secretos**
  - [x] `AAMN_CONTROL_TOKEN` definido y no commiteado.
  - [x] PSK definida vía `AAMN_PSK` o `AAMN_PSK_FILE` (no en JSON plano).
  - [x] `.gitignore` ya excluye `config.toml`, `.env` y ficheros de secretos.

- **Red**
  - [x] Puerto `9000` abierto si el nodo debe ser alcanzable desde Internet.
  - [x] Puertos `1080`, `50051`, `50052` limitados a `127.0.0.1` (ya en `docker-compose.yml`).
  - [x] Firewall del servidor solo permite lo necesario (p. ej. 80/443 y 9000).

- **Logs y privacidad**
  - [x] Logs de destinos SOCKS5 solo a nivel `debug` (no en `info` por defecto).
  - [x] No se loguean PSKs ni tokens.

- **Control de acceso**
  - [x] Acceso al servidor restringido (SSH con claves, sin usuarios innecesarios).
  - [x] Panel de control solo accesible a admins de confianza.

---

### 6. Flujo típico de despliegue

1. **Preparar servidor**
   - Instalar Docker + Docker Compose.
   - Configurar firewall (ufw, iptables, security group, etc.).
2. **Clonar repositorio**

   ```bash
   git clone <tu-fork-o-repo>.git
   cd Turnel
   ```

3. **Configurar secretos**
   - Crear archivo `.env` (no se sube a GitHub) con:

     ```bash
     AAMN_CONTROL_TOKEN="token-de-produccion"
     AAMN_PSK_FILE="/run/secrets/aamn_psk"
     ```

   - Usar `docker secrets` o mecanismos equivalentes del orquestador.

4. **Levantar servicios**

   ```bash
   docker compose --env-file .env up -d
   ```

5. **Verificar**
   - Comprobar logs del nodo: `docker compose logs -f aamn-node`.
   - Acceder a `http://<host>` y hacer login con el token.
   - Probar comandos de seguridad desde el dashboard.

---

### 7. Estado del Repositorio para GitHub

Este proyecto ya está preparado para subir a GitHub:

- `.gitignore` evita subir:
  - Binarios (`target/`, `aamn`, `aamn.exe`, librerías compartidas).
  - Logs (`*.log`).
  - Configuración sensible (`config.toml`, `secrets.json`, `.env*`).
- La configuración de producción se hace **por entorno**, no por archivos commiteados.
- La documentación está en:
  - `README.md` – visión general del proyecto.
  - `docs/SECURITY.md` – modelo de seguridad y limitaciones.
  - `docs/CONFIG.md` – guía de configuración.
  - `docs/PRODUCTION_AUDIT.md` y `PRODUCTION_AUDIT.md` – auditorías.
  - `ASPECTOS_LEGALES.md` – consideraciones legales.
  - `DEPLOYMENT.md` – este documento de despliegue.

Para publicar en GitHub:

```bash
git status           # confirmar que no hay secretos nuevos trackeados
git add .
git commit -m "Harden control API and add production deployment guide"
git push origin master
```

> Asegúrate de revisar el diff (`git diff`) antes de hacer `git add .` para confirmar que no estás incluyendo `.env`, PSKs o tokens reales.

