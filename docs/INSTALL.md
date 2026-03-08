# Guía de Instalación - AAMN

## Requisitos del Sistema

### Requisitos Mínimos
- **Sistema Operativo**: Windows 10+, Linux (Ubuntu 20.04+), macOS 11+
- **Memoria RAM**: 4 GB
- **Almacenamiento**: 500 MB libres
- **Red**: Conexión a Internet activa

### Requisitos Recomendados
- **Sistema Operativo**: Windows 11, Ubuntu 22.04+, macOS 13+
- **Memoria RAM**: 8 GB
- **Almacenamiento**: 1 GB libres
- **Red**: Conexión de banda ancha

## Instalación en Windows

### Método 1: Descarga Binaria

1. Ve a la página de releases en GitHub
2. Descarga el archivo `aamn-x.x.x-x86_64-pc-windows-msvc.zip`
3. Extrae el contenido en una carpeta de tu elección
4. Ejecuta `aamn.exe`

### Método 2: Compilación desde Código Fuente

```powershell
# Instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clonar el repositorio
git clone https://github.com/tu-usuario/aamn.git
cd aamn

# Compilar
cargo build --release

# Ejecutar
./target/release/aamn.exe
```

## Instalación en Linux

### Usando Cargo

```bash
# Instalar dependencias
sudo apt-get install build-essential pkg-config libssl-dev

# Instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clonar y compilar
git clone https://github.com/tu-usuario/aamn.git
cd aamn
cargo build --release

# Ejecutar
sudo setcap cap_net_admin+ep target/release/aamn
./target/release/aamn
```

### Permisos

Para crear interfaces TUN/TAP, necesitas permisos de administrador:

```bash
sudo groupadd tun
sudo usermod -a -G tun $USER
# Reiniciar sesión
```

## Instalación en macOS

```bash
# Instalar Xcode Command Line Tools
xcode-select --install

# Instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Compilar
git clone https://github.com/tu-usuario/aamn.git
cd aamn
cargo build --release

# Ejecutar
./target/release/aamn
```

## Configuración Inicial

### 1. Generar Identidad

```bash
aamn gen-identity --output identity.json
```

Esto generará:
- Clave pública (identidad del nodo)
- Clave privada (guárdala segura)

### 2. Configurar Puerto

Edita `config.toml`:

```toml
[network]
listen_addr = "0.0.0.0:9000"
```

### 3. Nodos Bootstrap

Para conectarte a la red inicialmente, especifica nodos bootstrap:

```bash
aamn start --bootstrap bootstrap1.aamn.net:9000
```

## Verificación de Instalación

```bash
# Verificar estado
aamn status

# Ver logs
tail -f aamn.log
```

## Solución de Problemas

### Error: "Permission denied" al crear TUN

```bash
# Linux: Agregar permisos
sudo setcap cap_net_admin+ep /ruta/a/aamn
```

### Error: "Address already in use"

Cambia el puerto en la configuración:

```toml
[network]
listen_addr = "0.0.0.0:9001"
```

### Error: Conexión fallida

1. Verifica tu conexión a Internet
2. Asegúrate de que el firewall permita el puerto
3. Prueba con nodos bootstrap diferentes

## Actualización

```bash
# Detener el nodo
aamn stop

# Actualizar código
git pull origin main

# Recompilar
cargo build --release

# Reiniciar
aamn start
```

## Desinstalación

### Windows
Elimina la carpeta donde extrajiste los archivos.

### Linux/macOS
```bash
cargo uninstall aamn
rm -rf ~/.aamn
