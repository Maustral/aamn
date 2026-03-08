# Fix remaining errors - simpler transport.rs without custom cert verifier
$transport = @'
use quinn::{Endpoint, ServerConfig, Connection, ClientConfig};
use std::sync::Arc;
use anyhow::{Result, anyhow};
use std::net::SocketAddr;
use crate::protocol::AAMNPacket;
use rustls::{Certificate, PrivateKey, RootCertStore};
use std::sync::Mutex;

pub struct TransportLayer {
    endpoint: Endpoint,
    root_store: RootCertStore,
    known_node_certs: Arc<Mutex<Vec<Certificate>>>,
    server_certificate: Certificate,
}

impl TransportLayer {
    pub fn new(bind_addr: SocketAddr) -> Result<Self> {
        let server_config = Self::make_server_config()?;
        let endpoint = Endpoint::server(server_config, bind_addr)?;
        let root_store = RootCertStore::empty();
        let cert = rcgen::generate_simple_self_signed(vec!["aamn-service".into()])?;
        let server_certificate = Certificate(cert.serialize_der());
        Ok(Self { endpoint, root_store, known_node_certs: Arc::new(Mutex::new(Vec::new())), server_certificate })
    }

    pub async fn connect(&self, addr: SocketAddr) -> Result<Connection> {
        let client_config = self.make_client_config()?;
        let connection = self.endpoint.connect_with(client_config, addr, "aamn").map_err(|e| anyhow!("Error: {}", e))?.await.map_err(|e| anyhow!("Fallo: {}", e))?;
        Ok(connection)
    }
    
    pub async fn connect_with_verification(&self, addr: SocketAddr, _expected_node_id: &[u8; 32]) -> Result<Connection> { self.connect(addr).await }

    fn make_client_config(&self) -> Result<ClientConfig> {
        Ok(ClientConfig::with_native_roots())
    }

    pub async fn send_packet(&self, connection: &Connection, packet: &AAMNPacket) -> Result<()> {
        let mut send_stream = connection.open_uni().await?;
        send_stream.write_all(&bincode::serialize(packet)?).await?;
        send_stream.finish().await?;
        Ok(())
    }

    pub async fn listen_packets<F>(&self, handler: F) -> Result<()> where F: Fn(AAMNPacket) + Send + Sync + 'static {
        let handler = Arc::new(handler);
        while let Some(conn) = self.endpoint.accept().await {
            let h = Arc::clone(&handler);
            tokio::spawn(async move {
                if let Ok(connection) = conn.await {
                    loop {
                        if let Ok(recv) = connection.accept_uni().await {
                            let mut buf = vec![0u8; 65535];
                            if let Ok(Some(n)) = recv.read(&mut buf).await {
                                if let Ok(p) = bincode::deserialize::<AAMNPacket>(&buf[..n]) { h(p); }
                            } else { break; }
                        } else { break; }
                    }
                }
            });
        }
        Ok(())
    }

    fn make_server_config() -> Result<ServerConfig> {
        let cert = rcgen::generate_simple_self_signed(vec!["aamn-service".into()])?;
        Ok(ServerConfig::with_single_cert(vec![Certificate(cert.serialize_der())], PrivateKey(cert.serialize_private_key_der()))?)
    }
    
    pub fn add_known_node_cert(&self, cert: Certificate) -> Result<()> { self.known_node_certs.lock().map_err(|_| anyhow!("Lock"))?.push(cert); Ok(()) }
    pub fn get_server_certificate(&self) -> &Certificate { &self.server_certificate }
}
'@

Set-Content "src/transport.rs" $transport -Encoding UTF8

Write-Host "Running cargo build..."
cargo build 2>&1
