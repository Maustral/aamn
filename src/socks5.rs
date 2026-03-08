//! SOCKS5 Proxy Server Implementation (RFC 1928)
//!
//! Expose a local TCP port that accepts standard SOCKS5 connections from
//! browsers (Firefox) and CLI apps (curl) and routes their streams transparently
//! through our AAMN onion circuits.

use anyhow::{anyhow, Result};
use std::net::{Ipv4Addr, Ipv6Addr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const SOCKS_VERSION: u8 = 0x05;

pub struct Socks5Server {
    port: u16,
}

impl Socks5Server {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    /// Inicia el bucle de escucha del proxy SOCKS5 en segundo plano (localhost)
    pub async fn start(&self) -> Result<()> {
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&addr).await?;
        tracing::info!("🧅 SOCKS5 Proxy listening on {}", addr);
        tracing::info!("   Try: curl --socks5 {} http://example.com", addr);

        loop {
            match listener.accept().await {
                Ok((mut socket, peer_addr)) => {
                    tracing::debug!("New SOCKS5 connection from {}", peer_addr);

                    // Manejar cada conexión SOCKS5 de forma concurrente
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_client(&mut socket).await {
                            tracing::error!("SOCKS5 client error ({}): {}", peer_addr, e);
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("SOCKS5 Accept error: {}", e);
                }
            }
        }
    }

    /// Procesa el handshake RFC 1928
    async fn handle_client(socket: &mut TcpStream) -> Result<()> {
        // 1. Handshake Initial: Cliente envía métodos de Auth (no auth por defecto)
        let mut header = [0u8; 2];
        socket.read_exact(&mut header).await?;

        if header[0] != SOCKS_VERSION {
            return Err(anyhow!("Unsupported SOCKS version: {}", header[0]));
        }

        let num_methods = header[1] as usize;
        let mut methods = vec![0u8; num_methods];
        socket.read_exact(&mut methods).await?;

        if !methods.contains(&0x00) {
            return Err(anyhow!("No supported authentication methods"));
        }

        // Responder aprobando "NO AUTHENTICATION REQUIRED" (0x00)
        socket.write_all(&[SOCKS_VERSION, 0x00]).await?;

        // 2. Client Connection Request
        let mut req_header = [0u8; 4];
        socket.read_exact(&mut req_header).await?;

        if req_header[0] != SOCKS_VERSION || req_header[1] != 0x01 {
            // Solo soportamos el comando CONNECT (0x01)
            socket
                .write_all(&[SOCKS_VERSION, 0x07, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
                .await?;
            return Err(anyhow!("Unsupported SOCKS command: {}", req_header[1]));
        }

        let address_type = req_header[3];
        let target_host: String;
        let target_port: u16;

        match address_type {
            0x01 => {
                // IPv4
                let mut ip_bytes = [0u8; 4];
                socket.read_exact(&mut ip_bytes).await?;
                let ip = Ipv4Addr::from(ip_bytes);
                target_host = ip.to_string();
            }
            0x03 => {
                // Domain Name
                let mut len_byte = [0u8; 1];
                socket.read_exact(&mut len_byte).await?;
                let len = len_byte[0] as usize;

                let mut domain_bytes = vec![0u8; len];
                socket.read_exact(&mut domain_bytes).await?;
                target_host = String::from_utf8_lossy(&domain_bytes).to_string();
            }
            0x04 => {
                // IPv6
                let mut ip_bytes = [0u8; 16];
                socket.read_exact(&mut ip_bytes).await?;
                let ip = Ipv6Addr::from(ip_bytes);
                target_host = ip.to_string();
            }
            _ => {
                socket
                    .write_all(&[SOCKS_VERSION, 0x08, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
                    .await?;
                return Err(anyhow!("Unsupported address type: {}", address_type));
            }
        }

        let mut port_bytes = [0u8; 2];
        socket.read_exact(&mut port_bytes).await?;
        target_port = u16::from_be_bytes(port_bytes);

        tracing::info!(
            "SOCKS5 Request to connect to: {}:{}",
            target_host,
            target_port
        );

        // Aceptar petición de forma exitosa localmente
        let reply_success = [
            SOCKS_VERSION,
            0x00,
            0x00,
            0x01,
            0x00,
            0x00,
            0x00,
            0x00, // IP binding 0.0.0.0
            0x00,
            0x00, // Puerto binding 0
        ];
        socket.write_all(&reply_success).await?;

        // 3. INTEGRACIÓN ONION ROUTING AAMN:
        // Aquí es donde en vez de hacer una conexión directa TcpStream::connect(),
        // tomamos los buffers I/O y los inyectamos en un circuito Onion AAMN hacia
        // un "Exit Node" (como en Tor).
        //
        // TEMPORAL (mientras agregamos los Exit Nodes al protocolo real):
        // Haremos un mock passthrough para testing local

        let target_addr = format!("{}:{}", target_host, target_port);
        match TcpStream::connect(&target_addr).await {
            Ok(mut remote) => {
                // Bi-directional copy entre SOCKS y Remote (passthrough temporal)
                let (mut client_recv, mut client_send) = socket.split();
                let (mut remote_recv, mut remote_send) = remote.split();

                let client_to_remote = tokio::io::copy(&mut client_recv, &mut remote_send);
                let remote_to_client = tokio::io::copy(&mut remote_recv, &mut client_send);

                let _ = tokio::try_join!(client_to_remote, remote_to_client);
                tracing::debug!("SOCKS5 Tunnel closed for {}:{}", target_host, target_port);
            }
            Err(e) => {
                tracing::error!(
                    "SOCKS5 Mock Exit Node Proxy failed to reach {}: {}",
                    target_addr,
                    e
                );
            }
        }

        Ok(())
    }
}
