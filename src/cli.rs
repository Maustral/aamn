//! AAMN CLI - Command Line Interface
//!
//! Interfaz de línea de comandos para el cliente/servidor AAMN

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Comando principal de AAMN
#[derive(Parser)]
#[command(name = "aamn")]
#[command(version = "0.2.0")]
#[command(about = "AAMN - Adaptive Anonymous Mesh Network", long_about = None)]
pub struct Cli {
    /// Nivel de verbosidad (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Archivo de configuración personalizado
    #[arg(short, long, default_value = None)]
    pub config: Option<PathBuf>,

    /// Ejecutar como daemon/servicio
    #[arg(short, long)]
    pub daemon: bool,

    /// Comando a ejecutar
    #[command(subcommand)]
    pub command: Commands,
}

/// Comandos disponibles
#[derive(Subcommand)]
pub enum Commands {
    /// Iniciar el nodo AAMN
    Start {
        /// Puerto de escucha
        #[arg(short, long, default_value = "9000")]
        port: u16,

        /// Dirección de bootstrap
        #[arg(long)]
        bootstrap: Option<String>,
    },

    /// Detener el nodo en ejecución
    Stop,

    /// Mostrar estado del nodo
    Status,

    /// Conectar a un peer
    Connect {
        /// Dirección del peer
        peer: String,
    },

    /// Listar peers conectados
    Peers,

    /// Generar nueva identidad
    GenIdentity {
        /// Archivo donde guardar la identidad
        #[arg(short, long, default_value = "identity.json")]
        output: PathBuf,
    },

    /// Validar configuración
    ValidateConfig {
        /// Archivo de configuración a validar
        #[arg(short, long, default_value = "config.toml")]
        config: PathBuf,
    },
}

impl Cli {
    /// Obtener nivel de log basado en verbose
    pub fn get_log_level(&self) -> &'static str {
        match self.verbose {
            0 => "info",
            1 => "debug",
            2 => "debug",
            _ => "trace",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_start() {
        let cli = Cli::parse_from(&["aamn", "start", "--port", "8080"]);
        match cli.command {
            Commands::Start { port, bootstrap } => {
                assert_eq!(port, 8080);
                assert_eq!(bootstrap, None);
            }
            _ => panic!("Expected Start command"),
        }
    }

    #[test]
    fn test_cli_parse_verbose() {
        let cli = Cli::parse_from(&["aamn", "-v", "status"]);
        assert_eq!(cli.verbose, 1);
    }

    #[test]
    fn test_log_level() {
        let cli = Cli::parse_from(&["aamn", "status"]);
        assert_eq!(cli.get_log_level(), "info");

        let cli = Cli::parse_from(&["aamn", "-v", "status"]);
        assert_eq!(cli.get_log_level(), "debug");
    }
}
