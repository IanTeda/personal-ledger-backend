//! Server configuration types and defaults.
//!
//! This module defines `ServerConfig` and related constants used by the
//! configuration loader. Values here represent programme defaults and are
//! typically overridden by an INI config file under `config/` or by
//! environment variables prefixed with `LEDGER_BACKEND`.

use crate::{telemetry, LedgerResult};
use std::path::PathBuf;

/// Default name for the configuration file (without extension).
///
/// This constant is combined with a `config/` directory and a `.conf`
/// extension to produce the default config file path (for example
/// `config/ledger-backend.conf`).
pub const CONFIG_FILE_NAME: &str = "ledger-backend";

/// Default server bind address used when no value is provided by file or env.
pub const DEFAULT_SERVER_ADDRESS: &str = "127.0.0.1";

/// Default server port used when no value is provided by file or env.
pub const DEFAULT_SERVER_PORT: u16 = 50059;

/// Default log level if none is specified.
pub const DEFAULT_LOG_LEVEL: telemetry::LogLevel = telemetry::LogLevel::WARN;

/// Default for whether TLS is enabled.
pub const DEFAULT_TLS_ENABLED: bool = false;

/// Defaults for TLS certificate path (none by default).
pub const DEFAULT_TLS_CERT_PATH: Option<&str> = None;

/// Defaults for TLS key path (none by default).
pub const DEFAULT_TLS_KEY_PATH: Option<&str> = None;



#[derive(Debug, Clone, serde::Deserialize)]
/// Server-specific configuration values.
///
/// `ServerConfig` holds the host/port the gRPC server will bind to and the
/// TLS settings required for encrypted connections. Fields are intentionally
/// simple to allow easy deserialisation from INI or environment sources.
pub struct ServerConfig {
    /// The IP address to bind the server to.
    /// Defaults to "127.0.0.1".
    pub address: String,

    /// The port number to bind the server on.
    /// Must be a valid port number (1023-65535).
    pub port: u16,

    /// The logging level for the application.
    ///
    /// If not set the configured default will be used. Values are provided
    /// via the `telemetry::LogLevel` enum (case-insensitive when deserialised).
    pub log_level: Option<telemetry::LogLevel>,

    /// Whether TLS encryption should be enabled for secure connections.
    /// When enabled, both `tls_cert_path` and `tls_key_path` must be provided.
    pub tls_enabled: bool,

    /// Path to the TLS certificate file in PEM format. Required when
    /// `tls_enabled` is true.
    pub tls_cert_path: Option<PathBuf>,

    /// Path to the TLS private key file in PEM format. Required when
    /// `tls_enabled` is true.
    pub tls_key_path: Option<PathBuf>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: DEFAULT_SERVER_ADDRESS.to_string(),
            port: DEFAULT_SERVER_PORT,
            log_level: Some(DEFAULT_LOG_LEVEL),
            tls_enabled: DEFAULT_TLS_ENABLED,
            tls_cert_path: DEFAULT_TLS_CERT_PATH.map(PathBuf::from),
            tls_key_path: DEFAULT_TLS_KEY_PATH.map(PathBuf::from),
        }
    }
}

impl ServerConfig {
    /// Build and return the bind `SocketAddr` for the server.
    ///
    /// This formats `server.address:server.port` and parses it into a
    /// `SocketAddr`. Returns an error if parsing fails.
    pub fn address(&self) -> LedgerResult<core::net::SocketAddr> {
        let address = format!("{}:{}", self.address, self.port);
        let address = address.parse()?;
        Ok(address)
    }

    /// Return the log level
    pub fn log_level(&self) -> telemetry::LogLevel {
        self.log_level.unwrap_or(DEFAULT_LOG_LEVEL)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use config::Config as ConfigLib;
    use std::net::SocketAddr;

    #[test]
    fn default_server_config_values() {
        let s = ServerConfig::default();
        assert_eq!(s.address, DEFAULT_SERVER_ADDRESS);
        assert_eq!(s.port, DEFAULT_SERVER_PORT);
        assert_eq!(s.tls_enabled, DEFAULT_TLS_ENABLED);
        assert!(s.tls_cert_path.is_none());
        assert!(s.tls_key_path.is_none());
    }

    #[test]
    fn server_address_parses_successfully() {
        let cfg = ServerConfig {
            address: "127.0.0.1".into(),
            port: 50051,
            log_level: None,
            tls_enabled: false,
            tls_cert_path: None,
            tls_key_path: None,
        };

        let addr = cfg.address().expect("address should parse");
        let expected: SocketAddr = "127.0.0.1:50051".parse().unwrap();
        assert_eq!(addr, expected);
    }

    #[test]
    fn server_address_parsing_fails_for_invalid_address() {
        let cfg = ServerConfig {
            address: "not_an_ip".into(),
            port: 80,
            log_level: None,
            tls_enabled: false,
            tls_cert_path: None,
            tls_key_path: None,
        };

        assert!(cfg.address().is_err(), "invalid address should return an error");
    }

    #[test]
    fn deserialize_from_configlib_with_defaults() {
        let cfglib = ConfigLib::builder()
            .set_default("address", DEFAULT_SERVER_ADDRESS)
            .unwrap()
            .set_default("port", DEFAULT_SERVER_PORT)
            .unwrap()
            .set_default("tls_enabled", DEFAULT_TLS_ENABLED)
            .unwrap()
            .build()
            .unwrap();

        let server_cfg: ServerConfig = cfglib.try_deserialize().expect("should deserialize");
        assert_eq!(server_cfg.address, DEFAULT_SERVER_ADDRESS);
        assert_eq!(server_cfg.port, DEFAULT_SERVER_PORT);
    assert_eq!(server_cfg.tls_enabled, DEFAULT_TLS_ENABLED);
    }

    #[test]
    fn server_config_default_is_valid_socket() {
        let s = ServerConfig::default();
        let combined = format!("{}:{}", s.address, s.port);
        assert!(
            combined.parse::<SocketAddr>().is_ok(),
            "default server address must parse to SocketAddr"
        );
    }
}