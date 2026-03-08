use anyhow::{anyhow, Result};
use tokio::sync::mpsc;

pub struct TunnelHandler {
    // Usar un puntero genérico sin especificar el tipo de Queue.
    // En una implementación real, usaríamos Box<dyn Device> o un patrón similar.
    device: Option<Box<dyn std::any::Any + Send + Sync>>,
}

impl TunnelHandler {
    pub fn new(name: &str) -> Result<Self> {
        let mut config = tun::Configuration::default();
        config
            .name(name)
            .address((10, 0, 0, 1)) // IP interna de la red AAMN
            .netmask((255, 255, 255, 0))
            .up();

        #[cfg(target_os = "windows")]
        config.platform(|_config| {
            // En Windows, tun-rs usa Wintun
            // _config.device_guid(Some(123456789));
        });

        #[allow(unused_assignments)]
        let device = match tun::create(&config) {
            Ok(dev) => Some(Box::new(dev) as Box<dyn std::any::Any + Send + Sync>),
            Err(e) => return Err(anyhow!("Fallo al crear interfaz TUN: {}", e)),
        };

        Ok(Self { device })
    }

    /// Lee paquetes de la interfaz virtual para ser procesados por AAMN.
    pub async fn run_capture(&self, tx: mpsc::Sender<Vec<u8>>) -> Result<()> {
        // Nota: tun-rs es síncrono por defecto, en una implementación real
        // usaríamos tokio::task::spawn_blocking o una versión asíncrona.
        let buf = [0u8; 2048];

        if self.device.is_none() {
            return Err(anyhow!("Dispositivo TUN no inicializado"));
        }

        loop {
            // Simular lectura (en prototipo)
            let packet = buf[..0].to_vec();
            if let Err(_e) = tx.send(packet).await {
                eprintln!("Error enviando paquete al procesador");
                return Ok(());
            }
        }
    }
}
