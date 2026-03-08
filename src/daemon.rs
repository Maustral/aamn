//! AAMN Daemon - Servicio del Sistema
//!
//! Implementación de daemon/servicio para ejecutar AAMN en background
//! con soporte para auto-inicio y gestión del sistema.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use thiserror::Error;

/// Errores del daemon
#[derive(Error, Debug)]
pub enum DaemonError {
    #[error("Error al iniciar el daemon: {0}")]
    StartError(String),

    #[error("Error al detener el daemon: {0}")]
    StopError(String),

    #[error("El daemon ya está en ejecución")]
    AlreadyRunning,

    #[error("El daemon no está en ejecución")]
    NotRunning,

    #[error("Error de IPC: {0}")]
    IpcError(String),
}

/// Estado del daemon
#[derive(Debug, Clone, PartialEq)]
pub enum DaemonState {
    ///Daemon no inicializado
    Idle,
    ///Daemon iniciando
    Starting,
    ///Daemon ejecutándose
    Running,
    ///Daemon deteniéndose
    Stopping,
    ///Daemon detenido
    Stopped,
    ///Error en el daemon
    Error(String),
}

/// Información del proceso daemon
#[derive(Debug, Clone)]
pub struct DaemonInfo {
    /// PID del proceso
    pub pid: u32,
    /// Estado actual
    pub state: DaemonState,
    /// Uptime en segundos
    pub uptime: u64,
    /// Puerto de escucha
    pub port: u16,
    /// Número de peers conectados
    pub peers: usize,
}

impl Default for DaemonInfo {
    fn default() -> Self {
        Self {
            pid: 0,
            state: DaemonState::Idle,
            uptime: 0,
            port: 9000,
            peers: 0,
        }
    }
}

/// Administrador del daemon AAMN
pub struct DaemonManager {
    /// Estado actual del daemon
    state: Arc<Mutex<DaemonState>>,
    /// Información del daemon
    info: Arc<Mutex<DaemonInfo>>,
    /// PID del proceso hijo (si es daemon)
    child_pid: Arc<Mutex<Option<u32>>>,
    /// Socket de control (para comunicación)
    control_socket: PathBuf,
}

impl DaemonManager {
    /// Crear un nuevo administrador de daemon
    pub fn new() -> Self {
        let socket_path = if let Some(proj_dirs) = directories::ProjectDirs::from("com", "aamn", "AAMN") {
            proj_dirs.data_local_dir().join("daemon.sock")
        } else {
            PathBuf::from("daemon.sock")
        };

        Self {
            state: Arc::new(Mutex::new(DaemonState::Idle)),
            info: Arc::new(Mutex::new(DaemonInfo::default())),
            child_pid: Arc::new(Mutex::new(None)),
            control_socket: socket_path,
        }
    }

    /// Iniciar el daemon
    pub async fn start(&self, port: u16, daemonize: bool) -> Result<DaemonInfo, DaemonError> {
        // Verificar si ya está corriendo
        {
            let state = self.state.lock().await;
            if *state == DaemonState::Running {
                return Err(DaemonError::AlreadyRunning);
            }
        }

        // Actualizar estado
        {
            let mut state = self.state.lock().await;
            *state = DaemonState::Starting;
        }

        if daemonize {
            // Ejecutar en background (daemonize)
            self.start_daemonized(port).await?;
        } else {
            // Ejecutar en foreground
            self.start_foreground(port).await?;
        }

        // Obtener info
        let info = self.info().await?;
        Ok(info)
    }

    /// Iniciar en modo daemon (background)
    async fn start_daemonized(&self, port: u16) -> Result<(), DaemonError> {
        // En Unix usaríamos fork(), pero en Windows usamos un hilo
        // El daemon real en Windows usaría CreateService
        
        // Por ahora, ejecutamos en un hilo separado
        let state = self.state.clone();
        let info = self.info.clone();
        
        tokio::spawn(async move {
            let mut info_lock = info.lock().await;
            info_lock.state = DaemonState::Running;
            info_lock.port = port;
            info_lock.pid = std::process::id();
            info_lock.uptime = 0;
            drop(info_lock);

            let mut state_lock = state.lock().await;
            *state_lock = DaemonState::Running;
            drop(state_lock);

            // Mantener el daemon vivo
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                
                let state_lock = state.lock().await;
                if *state_lock == DaemonState::Stopping {
                    break;
                }
                drop(state_lock);

                let mut info_lock = info.lock().await;
                info_lock.uptime += 1;
            }
        });

        Ok(())
    }

    /// Iniciar en modo foreground
    async fn start_foreground(&self, port: u16) -> Result<(), DaemonError> {
        let state = self.state.clone();
        let info = self.info.clone();

        // Actualizar info
        {
            let mut info_lock = info.lock().await;
            info_lock.state = DaemonState::Running;
            info_lock.port = port;
            info_lock.pid = std::process::id();
        }

        {
            let mut state_lock = state.lock().await;
            *state_lock = DaemonState::Running;
        }

        Ok(())
    }

    /// Detener el daemon
    pub async fn stop(&self) -> Result<(), DaemonError> {
        // Verificar estado
        {
            let state = self.state.lock().await;
            if *state != DaemonState::Running {
                return Err(DaemonError::NotRunning);
            }
        }

        // Actualizar estado
        {
            let mut state = self.state.lock().await;
            *state = DaemonState::Stopping;
        }

        // Enviar señal de terminación al proceso hijo
        {
            let pid = self.child_pid.lock().await;
            if let Some(child_pid) = *pid {
                #[cfg(windows)]
                {
                    use std::process::Command;
                    let _ = Command::new("taskkill")
                        .args(&["/PID", &child_pid.to_string()])
                        .output();
                }
            }
        }

        // Actualizar estado final
        {
            let mut state = self.state.lock().await;
            *state = DaemonState::Stopped;
        }

        Ok(())
    }

    /// Obtener estado del daemon
    pub async fn state(&self) -> DaemonState {
        let state = self.state.lock().await;
        state.clone()
    }

    /// Obtener información del daemon
    pub async fn info(&self) -> Result<DaemonInfo, DaemonError> {
        let info = self.info.lock().await;
        Ok(info.clone())
    }

    /// Verificar si el daemon está corriendo
    pub async fn is_running(&self) -> bool {
        let state = self.state.lock().await;
        *state == DaemonState::Running
    }

    /// Reiniciar el daemon
    pub async fn restart(&self, port: u16) -> Result<DaemonInfo, DaemonError> {
        self.stop().await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        self.start(port, false).await
    }
}

impl Default for DaemonManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_daemon_manager_new() {
        let manager = DaemonManager::new();
        let state = manager.state().await;
        assert_eq!(state, DaemonState::Idle);
    }

    #[tokio::test]
    async fn test_daemon_start_stop() {
        let manager = DaemonManager::new();
        
        // Iniciar
        let info = manager.start(9000, false).await;
        assert!(info.is_ok());
        
        // Verificar que está corriendo
        assert!(manager.is_running().await);
        
        // Detener
        let result = manager.stop().await;
        assert!(result.is_ok());
    }
}

