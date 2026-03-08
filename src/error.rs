//! AAMN Error - Sistema de Gestión de Errores
//!
//! Sistema centralizado de errores para AAMN usando thiserror

use thiserror::Error;

/// Errores principales del proyecto AAMN
#[derive(Error, Debug)]
pub enum AAMNError {
    // ============================================
    // Errores de Criptografía
    // ============================================
    #[error("Error criptográfico: {0}")]
    Crypto(String),

    #[error("Error en el intercambio de claves: {0}")]
    KeyExchange(String),

    #[error("Fallo en el cifrado: {0}")]
    EncryptionError(String),

    #[error("Fallo en el descifrado: {0}")]
    DecryptionError(String),

    #[error("Clave inválida: {0}")]
    InvalidKey(String),

    // ============================================
    // Errores de Red
    // ============================================
    #[error("Error de red: {0}")]
    Network(String),

    #[error("Conexión rechazada: {0}")]
    ConnectionRefused(String),

    #[error("Timeout de conexión: {0}")]
    ConnectionTimeout(String),

    #[error("Error de transporte: {0}")]
    Transport(String),

    #[error("Puerto en uso: {0}")]
    PortInUse(String),

    // ============================================
    // Errores de Routing
    // ============================================
    #[error("Error de enrutamiento: {0}")]
    Routing(String),

    #[error("Ruta no encontrada")]
    RouteNotFound,

    #[error("Nodo no disponible: {0}")]
    NodeUnavailable(String),

    #[error("No hay nodos disponibles")]
    NoNodesAvailable,

    // ============================================
    // Errores de Configuración
    // ============================================
    #[error("Error de configuración: {0}")]
    Config(String),

    #[error("Archivo de configuración no encontrado: {0}")]
    ConfigFileNotFound(String),

    #[error("Configuración inválida: {0}")]
    InvalidConfig(String),

    // ============================================
    // Errores de Persistencia
    // ============================================
    #[error("Error de persistencia: {0}")]
    Storage(String),

    #[error("Archivo no encontrado: {0}")]
    FileNotFound(String),

    #[error("Error de permisos: {0}")]
    PermissionDenied(String),

    // ============================================
    // Errores de Autenticación
    // ============================================
    #[error("Error de autenticación: {0}")]
    Auth(String),

    #[error("Firma inválida")]
    InvalidSignature,

    #[error("Certificado inválido: {0}")]
    InvalidCertificate(String),

    // ============================================
    // Errores de Protocolo
    // ============================================
    #[error("Error de protocolo: {0}")]
    Protocol(String),

    #[error("Versión de protocolo incompatible: {0}")]
    IncompatibleVersion(String),

    #[error("Paquete malformado: {0}")]
    MalformedPacket(String),

    // ============================================
    // Errores de Sistema
    // ============================================
    #[error("Error del sistema: {0}")]
    System(String),

    #[error("Recursos insuficientes: {0}")]
    InsufficientResources(String),

    #[error("Daemon no disponible")]
    DaemonNotAvailable,

    #[error("Daemon ya en ejecución")]
    DaemonAlreadyRunning,

    // ============================================
    // Errores Desconocidos
    // ============================================
    #[error("Error desconocido: {0}")]
    Unknown(String),
}

/// Result tipo para funciones que pueden fallar
pub type Result<T> = std::result::Result<T, AAMNError>;

/// Extensiones adicionales para el manejo de errores
impl AAMNError {
    /// Obtener el código de error para logging
    pub fn code(&self) -> &'static str {
        match self {
            // Criptografía
            Self::Crypto(_) => "CRYPTO",
            Self::KeyExchange(_) => "KEY_EXCHANGE",
            Self::EncryptionError(_) => "ENCRYPT",
            Self::DecryptionError(_) => "DECRYPT",
            Self::InvalidKey(_) => "INVALID_KEY",

            // Red
            Self::Network(_) => "NETWORK",
            Self::ConnectionRefused(_) => "CONN_REFUSED",
            Self::ConnectionTimeout(_) => "CONN_TIMEOUT",
            Self::Transport(_) => "TRANSPORT",
            Self::PortInUse(_) => "PORT_IN_USE",

            // Routing
            Self::Routing(_) => "ROUTING",
            Self::RouteNotFound => "ROUTE_NOT_FOUND",
            Self::NodeUnavailable(_) => "NODE_UNAVAILABLE",
            Self::NoNodesAvailable => "NO_NODES",

            // Configuración
            Self::Config(_) => "CONFIG",
            Self::ConfigFileNotFound(_) => "CONFIG_NOT_FOUND",
            Self::InvalidConfig(_) => "INVALID_CONFIG",

            // Persistencia
            Self::Storage(_) => "STORAGE",
            Self::FileNotFound(_) => "FILE_NOT_FOUND",
            Self::PermissionDenied(_) => "PERMISSION",

            // Autenticación
            Self::Auth(_) => "AUTH",
            Self::InvalidSignature => "INVALID_SIG",
            Self::InvalidCertificate(_) => "INVALID_CERT",

            // Protocolo
            Self::Protocol(_) => "PROTOCOL",
            Self::IncompatibleVersion(_) => "VERSION",
            Self::MalformedPacket(_) => "MALFORMED",

            // Sistema
            Self::System(_) => "SYSTEM",
            Self::InsufficientResources(_) => "RESOURCES",
            Self::DaemonNotAvailable => "DAEMON_DOWN",
            Self::DaemonAlreadyRunning => "DAEMON_RUNNING",

            // Unknown
            Self::Unknown(_) => "UNKNOWN",
        }
    }

    /// Verificar si es un error recuperable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Network(_)
                | Self::ConnectionTimeout(_)
                | Self::NodeUnavailable(_)
                | Self::NoNodesAvailable
                | Self::Transport(_)
        )
    }

    /// Verificar si es un error crítico
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            Self::Crypto(_)
                | Self::KeyExchange(_)
                | Self::InvalidKey(_)
                | Self::InvalidSignature
                | Self::InvalidCertificate(_)
        )
    }
}

/// Implementación para convertir errores de libraries externas
impl From<std::io::Error> for AAMNError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => AAMNError::FileNotFound(err.to_string()),
            std::io::ErrorKind::PermissionDenied => AAMNError::PermissionDenied(err.to_string()),
            _ => AAMNError::System(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for AAMNError {
    fn from(err: serde_json::Error) -> Self {
        AAMNError::Config(err.to_string())
    }
}

impl From<toml::de::Error> for AAMNError {
    fn from(err: toml::de::Error) -> Self {
        AAMNError::Config(err.to_string())
    }
}

impl From<toml::ser::Error> for AAMNError {
    fn from(err: toml::ser::Error) -> Self {
        AAMNError::Config(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let err = AAMNError::Network("test".to_string());
        assert_eq!(err.code(), "NETWORK");

        let err = AAMNError::RouteNotFound;
        assert_eq!(err.code(), "ROUTE_NOT_FOUND");
    }

    #[test]
    fn test_error_recoverable() {
        assert!(AAMNError::Network("test".to_string()).is_recoverable());
        assert!(!AAMNError::Crypto("test".to_string()).is_recoverable());
    }

    #[test]
    fn test_error_critical() {
        assert!(AAMNError::InvalidKey("test".to_string()).is_critical());
        assert!(!AAMNError::Network("test".to_string()).is_critical());
    }
}
