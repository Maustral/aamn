//! AAMN Logging - Sistema de Logging Estructurado
//!
//! Sistema de logging usando tracing para logs estructurados
//! con rotación de archivos y salida JSON.
//!
//! ✅ FASE 3: Logging de Seguridad implementado

use std::path::PathBuf;
use tracing::Level;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// ✅ FASE 3: Niveles de logging de seguridad
#[derive(Debug, Clone)]
pub enum SecurityLevel {
    /// Evento de seguridad informativo
    Info,
    /// Advertencia de seguridad
    Warning,
    /// Evento crítico de seguridad
    Critical,
}

/// ✅ FASE 3: Funciones de logging de seguridad
///
/// Registra un intento de conexión fallido
pub fn log_connection_failed(addr: &str, reason: &str) {
    tracing::warn!(
        target: "security",
        event = "connection_failed",
        address = addr,
        reason = reason,
        "Intento de conexión fallido"
    );
}

/// Registra un error de autenticación
pub fn log_auth_failure(node_id: Option<&[u8; 32]>, reason: &str) {
    tracing::warn!(
        target: "security",
        event = "auth_failure",
        node_id = node_id.map(hex::encode),
        reason = reason,
        "Error de autenticación"
    );
}

/// Registra una anomalía de tráfico detectada
pub fn log_traffic_anomaly(description: &str, details: &str) {
    tracing::warn!(
        target: "security",
        event = "traffic_anomaly",
        description = description,
        details = details,
        "Anomalía de tráfico detectada"
    );
}

/// Registra un cambio de estado de un nodo
pub fn log_node_state_change(node_id: &[u8; 32], old_state: &str, new_state: &str) {
    tracing::info!(
        target: "security",
        event = "node_state_change",
        node_id = hex::encode(node_id),
        old_state = old_state,
        new_state = new_state,
        "Cambio de estado de nodo"
    );
}

/// Registra un evento crítico de seguridad
pub fn log_security_critical(event_type: &str, description: &str) {
    tracing::error!(
        target: "security",
        event = event_type,
        description = description,
        "Evento crítico de seguridad"
    );
}

/// Registra un intento de rate limiting
pub fn log_rate_limit_exceeded(node_id: &[u8; 32], current_rate: u32, limit: u32) {
    tracing::warn!(
        target: "security",
        event = "rate_limit_exceeded",
        node_id = hex::encode(node_id),
        current_rate = current_rate,
        limit = limit,
        "Rate limit excedido"
    );
}

/// Configuración del sistema de logging
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Nivel mínimo de logging
    pub level: Level,
    /// Habilitar logs JSON
    pub json: bool,
    /// Directorio de logs
    pub directory: Option<PathBuf>,
    /// Nombre del archivo de log
    pub filename: Option<String>,
    /// Máximo tamaño de archivo (MB)
    pub max_file_size: u64,
    /// Número máximo de archivos de log
    pub max_files: usize,
    /// Habilitar rotación de logs
    pub rotation: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            json: false,
            directory: None,
            filename: Some("aamn.log".to_string()),
            max_file_size: 10,
            max_files: 5,
            rotation: true,
        }
    }
}

/// Inicializa el sistema de logging
pub fn init(config: &LoggingConfig) -> Result<(), Box<dyn std::error::Error>> {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,aamn=debug"));

    // Determinar el directorio de logs
    let log_dir = config.directory.clone().unwrap_or_else(|| {
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "aamn", "AAMN") {
            proj_dirs.data_local_dir().to_path_buf()
        } else {
            PathBuf::from(".")
        }
    });

    if !log_dir.exists() {
        std::fs::create_dir_all(&log_dir)?;
    }

    // Crear el writer para archivos
    let file_appender = tracing_appender::rolling::RollingFileAppender::new(
        tracing_appender::rolling::Rotation::DAILY,
        &log_dir,
        config.filename.as_deref().unwrap_or("aamn.log"),
    );

    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Mantener el guard vivo durante toda la aplicación
    std::mem::forget(_guard);

    // Inicializar según el formato seleccionado
    if config.json {
        let subscriber = tracing_subscriber::registry().with(env_filter).with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_target(true)
                .json(),
        );
        subscriber.init();
    } else {
        let subscriber = tracing_subscriber::registry().with(env_filter).with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_target(true),
        );
        subscriber.init();
    }

    tracing::info!("Logging inicializado en {}", log_dir.display());
    Ok(())
}

/// Macro para logging rápido
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        tracing::error!($($arg)*)
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*)
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*)
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        tracing::debug!($($arg)*)
    };
}

#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        tracing::trace!($($arg)*)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, Level::INFO);
        assert!(!config.json);
    }

    #[test]
    fn test_init_logging_to_stdout() {
        let config = LoggingConfig {
            level: Level::DEBUG,
            json: false,
            directory: None,
            filename: Some("test.log".to_string()),
            rotation: false,
            max_file_size: 10,
            max_files: 5,
        };

        // Usar stdout como writer para tests
        let subscriber = tracing_subscriber::registry()
            .with(EnvFilter::new("info"))
            .with(fmt::layer().with_target(true));

        let _ = subscriber.try_init();
    }
}
