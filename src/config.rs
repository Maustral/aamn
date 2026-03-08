//! Módulo de Configuración - AAMN
//!
//! Sistema de configuración centralizado para nodos AAMN.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct Config {
    pub network: NetworkConfig,
    pub security: SecurityConfig,
    pub performance: PerformanceConfig,
    pub logging: LoggingConfig,
}


impl Config {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&contents)?;
        Ok(config)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(path, contents)?;
        Ok(())
    }

    pub fn load_from_env() -> Self {
        Self {
            network: NetworkConfig::load_from_env(),
            security: SecurityConfig::load_from_env(),
            performance: PerformanceConfig::load_from_env(),
            logging: LoggingConfig::load_from_env(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        self.network.validate()?;
        self.security.validate()?;
        self.performance.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub listen_addr: SocketAddr,
    pub bootstrap_node: Option<SocketAddr>,
    pub use_quic: bool,
    pub max_connections: u32,
    pub connection_timeout: u64,
    pub nat_traversal: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:9000".parse().unwrap(),
            bootstrap_node: None,
            use_quic: true,
            max_connections: 100,
            connection_timeout: 30,
            nat_traversal: false,
        }
    }
}

impl NetworkConfig {
    pub fn load_from_env() -> Self {
        use std::env;
        let mut config = Self::default();
        if let Ok(addr) = env::var("AAMN_LISTEN_ADDR") {
            if let Ok(parsed) = addr.parse() {
                config.listen_addr = parsed;
            }
        }
        config
    }

    pub fn validate(&self) -> Result<()> {
        if self.max_connections == 0 {
            return Err(anyhow!("max_connections debe ser mayor que 0"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_onion_encryption: bool,
    pub onion_layers: usize,
    pub enable_hmac: bool,
    pub enable_pow: bool,
    pub pow_difficulty: u32,
    pub enable_chaff: bool,
    pub chaff_probability: f32,
    pub min_route_length: usize,
    pub max_route_length: usize,
    pub psk: Option<String>,
    pub psk_file: Option<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_onion_encryption: true,
            onion_layers: 3,
            enable_hmac: true,
            enable_pow: false,
            pow_difficulty: 20,
            enable_chaff: true,
            chaff_probability: 0.1,
            min_route_length: 2,
            max_route_length: 5,
            psk: None,
            psk_file: None,
        }
    }
}

impl SecurityConfig {
    pub fn load_from_env() -> Self {
        Self::default()
    }
    pub fn validate(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub max_packet_size: usize,
    pub fragment_size: usize,
    pub enable_rate_limiting: bool,
    pub rate_limit_rps: u32,
    pub enable_metrics: bool,
    pub metrics_interval: u64,
    pub send_buffer_size: usize,
    pub recv_buffer_size: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_packet_size: 1450,
            fragment_size: 512,
            enable_rate_limiting: true,
            rate_limit_rps: 100,
            enable_metrics: true,
            metrics_interval: 60,
            send_buffer_size: 65536,
            recv_buffer_size: 65536,
        }
    }
}

impl PerformanceConfig {
    pub fn load_from_env() -> Self {
        Self::default()
    }
    pub fn validate(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_enabled: bool,
    pub file_path: String,
    pub stdout_enabled: bool,
    pub rotation: LogRotation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogRotation {
    Never,
    Daily,
    Weekly,
    Size(u64),
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file_enabled: false,
            file_path: "aamn.log".to_string(),
            stdout_enabled: true,
            rotation: LogRotation::Daily,
        }
    }
}

impl LoggingConfig {
    pub fn load_from_env() -> Self {
        Self::default()
    }
}

pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }
    pub fn with_network(mut self, network: NetworkConfig) -> Self {
        self.config.network = network;
        self
    }
    pub fn with_security(mut self, security: SecurityConfig) -> Self {
        self.config.security = security;
        self
    }
    pub fn with_performance(mut self, performance: PerformanceConfig) -> Self {
        self.config.performance = performance;
        self
    }
    pub fn with_logging(mut self, logging: LoggingConfig) -> Self {
        self.config.logging = logging;
        self
    }
    pub fn build(self) -> Result<Config> {
        self.config.validate()?;
        Ok(self.config)
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.network.listen_addr.port() == 9000);
    }
}
