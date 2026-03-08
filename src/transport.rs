use crate::protocol::AAMNPacket;
use anyhow::{anyhow, Result};
use quinn::{ClientConfig, Connection, Endpoint, ServerConfig};
use rustls::{Certificate, PrivateKey, RootCertStore};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;

pub struct TransportLayer {
    endpoint: Endpoint,
    #[allow(dead_code)]
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
        let server_certificate = Certificate(cert.serialize_der()?);
        Ok(Self {
            endpoint,
            root_store,
            known_node_certs: Arc::new(Mutex::new(Vec::new())),
            server_certificate,
        })
    }

    pub async fn connect(&self, addr: SocketAddr) -> Result<Connection> {
        let client_config = self.make_client_config()?;
        let connection = self
            .endpoint
            .connect_with(client_config, addr, "aamn")
            .map_err(|e| anyhow!("Error: {}", e))?
            .await
            .map_err(|e| anyhow!("Fallo: {}", e))?;
        Ok(connection)
    }

    pub async fn connect_with_verification(
        &self,
        addr: SocketAddr,
        _expected_node_id: &[u8; 32],
    ) -> Result<Connection> {
        // En una versión futura, este método verificará que el certificado
        // remoto esté vinculado a la identidad esperada del nodo. Por ahora,
        // reutilizamos `connect` pero mantenemos la firma para no romper la API.
        self.connect(addr).await
    }

    fn make_client_config(&self) -> Result<ClientConfig> {
        // En lugar de confiar en el almacén de roots del sistema, limitamos
        // explícitamente la confianza a los certificados conocidos si están
        // configurados, manteniendo compatibilidad hacia atrás.
        let mut roots = self.root_store.clone();
        {
            let known = self.known_node_certs.lock().map_err(|_| anyhow!("Lock"))?;
            for cert in known.iter() {
                roots.add(cert).map_err(|_| anyhow!("Invalid cert"))?;
            }
        }

        let client_config = ClientConfig::with_root_certificates(roots);
        Ok(client_config)
    }

    pub async fn send_packet(&self, connection: &Connection, packet: &AAMNPacket) -> Result<()> {
        let mut send_stream = connection.open_uni().await?;
        send_stream.write_all(&bincode::serialize(packet)?).await?;
        send_stream.finish().await?;
        Ok(())
    }

    pub async fn listen_packets<F>(&self, handler: F) -> Result<()>
    where
        F: Fn(AAMNPacket) + Send + Sync + 'static,
    {
        let handler = Arc::new(handler);
        while let Some(conn) = self.endpoint.accept().await {
            let h = Arc::clone(&handler);
            tokio::spawn(async move {
                if let Ok(connection) = conn.await {
                    while let Ok(mut recv) = connection.accept_uni().await {
                        let mut buf = vec![0u8; 65535];
                        if let Ok(Some(n)) = recv.read(&mut buf).await {
                            if let Ok(p) = bincode::deserialize::<AAMNPacket>(&buf[..n]) {
                                h(p);
                            }
                        }
                    }
                }
            });
        }
        Ok(())
    }

    fn make_server_config() -> Result<ServerConfig> {
        let cert = rcgen::generate_simple_self_signed(vec!["aamn-service".into()])?;
        Ok(ServerConfig::with_single_cert(
            vec![Certificate(cert.serialize_der()?)],
            PrivateKey(cert.serialize_private_key_der()),
        )?)
    }

    pub fn add_known_node_cert(&self, cert: Certificate) -> Result<()> {
        self.known_node_certs
            .lock()
            .map_err(|_| anyhow!("Lock"))?
            .push(cert);
        Ok(())
    }
    pub fn get_server_certificate(&self) -> &Certificate {
        &self.server_certificate
    }
}
