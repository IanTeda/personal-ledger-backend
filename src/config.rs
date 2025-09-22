use config::{Config as ConfigLib, ConfigError as ConfigLibError};
use std::path::PathBuf;

use crate::{telemetry, LedgerResult};

/// Default name for the configuration file (without extension).
///
/// This is combined with a `config/` directory and a `.conf` extension
/// to produce the default config file path (e.g. `config/ledger-backend.conf`).
const CONFIG_FILE_NAME: &str = "ledger-backend";

/// Environment variable prefix used for overriding configuration values.
///
/// Environment variables are expected in the form `LEDGER_BACKEND_<SECTION>_<KEY>`.
const ENV_SUFFIX: &str = "LEDGER_BACKEND";

/// Default server bind address used when no value is provided by file or env.
const DEFAULT_SERVER_ADDRESS: &str = "127.0.0.1";

/// Default server port used when no value is provided by file or env.
const DEFAULT_SERVER_PORT: u16 = 50059;

/// Default log level if none is specified.
const DEFAULT_LOG_LEVEL: telemetry::LogLevel = telemetry::LogLevel::WARN;

/// Default for whether TLS is enabled.
const DEFAULT_TLS_ENABLED: bool = false;

/// Defaults for TLS certificate path (none by default).
const DEFAULT_TLS_CERT_PATH: Option<&str> = None;

/// Defaults for TLS key path (none by default).
const DEFAULT_TLS_KEY_PATH: Option<&str> = None;


#[derive(thiserror::Error, Debug)]
/// Errors produced while loading or validating configuration.
///
/// This enum wraps errors from the underlying `config` crate and adds
/// domain-specific validation variants such as an invalid server address.
pub enum ConfigError {
    /// Error from the underlying config crate during file loading or parsing.
    ///
    /// This wraps errors from the `config` crate such as file not found,
    /// invalid syntax, or parsing failures.
    #[error("Configuration parsing error: {0}")]
    Parsing(#[from] ConfigLibError),

    /// Validation error with a descriptive message.
    ///
    /// This is used for configuration validation failures such as invalid
    /// values, missing required fields, or security misconfigurations.
    #[error("Invalid configuration: {0}")]
    Validation(String),

    /// Error indicating an invalid server address format.
    ///
    /// This is used for configuration validation failures such as invalid
    /// values, missing required fields, or security misconfigurations.
    #[error("Invalid server address: {0}")]
    InvalidServerAddress(#[from] std::net::AddrParseError),
}

/// Main application configuration structure.
///
/// `LedgerConfig` represents the top-level configuration that is deserialized
/// from configuration files, environment variables, or defaults. It currently
/// contains only the `server` section which controls network binding and TLS.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
pub struct LedgerConfig {
    /// Server configuration settings.
    pub server: ServerConfig,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
/// Server-specific configuration values.
///
/// This struct controls the host/port where the gRPC server binds, plus
/// TLS-related options (certificate and key file paths).
pub struct ServerConfig {
    /// The IP address to bind the server to.
    /// Defaults to "127.0.0.1".
    pub address: String,

    /// The port number to bind the server on.
    /// Must be a valid port number (1023-65535).
    pub port: u16,

    /// The logging level for the application.
    /// If not set, defaults to `INFO`.
    /// Acceptable values: OFF, ERROR, WARN, INFO, DEBUG, TRACE.
    pub log_level: Option<telemetry::LogLevel>,

    /// Whether TLS encryption should be enabled for secure connections.
    /// When enabled, both `tls_cert_path` and `tls_key_path` must be provided.
    pub tls_enabled: bool,

    /// Path to the TLS certificate file in PEM format.
    /// Required when `tls_enabled` is true.
    pub tls_cert_path: Option<PathBuf>,

    /// Path to the TLS private key file in PEM format.
    /// Required when `tls_enabled` is true.
    pub tls_key_path: Option<PathBuf>,
}


impl LedgerConfig {
    /// Load the application configuration.
    ///
    /// Loads configuration from three sources in order of increasing priority:
    /// 1. Defaults (lowest priority)
    /// 2. Optional configuration file at `config/<CONFIG_FILE_NAME>.conf` (INI format)
    /// 3. Environment variables prefixed with `ENV_SUFFIX` (highest priority)
    ///
    /// If the config file is missing this function logs a warning and continues
    /// using defaults and environment variables.
    pub fn parse() -> Result<LedgerConfig, ConfigError> {
        // Get the directory that the binary is being run from
        let binary_path = std::env::current_dir()
            .expect("Failed to determine the current directory");

        // Set the configuration directory for the app
        let config_directory = binary_path.join("config");

        // Set the configuration file name to be the package name with .conf extension
        let config_filename = format!("{}.conf", CONFIG_FILE_NAME);

        // Set the default config file path
        let config_file_path = config_directory.join(config_filename);

        // Start with defaults (lowest priority)
        let mut builder = ConfigLib::builder()
            .set_default("server.address", DEFAULT_SERVER_ADDRESS)?
            .set_default("server.port", DEFAULT_SERVER_PORT)?
            .set_default("server.tls_enabled", DEFAULT_TLS_ENABLED)?;

        // If the config file exists, load it (overrides defaults). If not, warn and continue with defaults
        if config_file_path.exists() {
            // The repository contains `config/ledger-backend.conf` as an INI-style
            // file. Explicitly load the file as INI so the parser accepts `.conf`.
            builder = builder.add_source(
                config::File::from(config_file_path).format(config::FileFormat::Ini)
            );
        } else {
            tracing::warn!("Config file '{}' not found; using defaults and environment variables", config_file_path.display());
        }

        // Finally add environment variables (highest priority)
        builder = builder.add_source(
            config::Environment::with_prefix(ENV_SUFFIX)
                .prefix_separator("_")
                .separator("_"),
        );

        let config = builder.build()?;

        // Deserialize the generic `config::Config` into our `LedgerConfig` domain type.
        let ledger_config: LedgerConfig = config.try_deserialize()?;

        // Validate the loaded configuration
        // ledger_config.validate()?;

        Ok(ledger_config)
    }

    /// Build and return the bind `SocketAddr` for the server.
    ///
    /// This formats `server.address:server.port` and parses it into a
    /// `SocketAddr`. Returns an error if parsing fails.
    pub fn server_address(&self) -> LedgerResult<core::net::SocketAddr> {
        let address = format!("{}:{}", self.server.address, self.server.port);
        let address = address.parse()?;
        Ok(address)
    }

    pub fn log_level(&self) -> telemetry::LogLevel {
        self.server.log_level.unwrap_or(DEFAULT_LOG_LEVEL)
    }
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
        let cfg = LedgerConfig {
            server: ServerConfig {
                address: "127.0.0.1".into(),
                port: 50051,
                log_level: None,
                tls_enabled: false,
                tls_cert_path: None,
                tls_key_path: None,
            },
        };

        let addr = cfg.server_address().expect("address should parse");
        let expected: SocketAddr = "127.0.0.1:50051".parse().unwrap();
        assert_eq!(addr, expected);
    }

    #[test]
    fn server_address_parsing_fails_for_invalid_address() {
        let cfg = LedgerConfig {
            server: ServerConfig {
                address: "not_an_ip".into(),
                port: 80,
                log_level: None,
                tls_enabled: false,
                tls_cert_path: None,
                tls_key_path: None,
            },
        };

        assert!(cfg.server_address().is_err(), "invalid address should return an error");
    }

    #[test]
    fn deserialize_from_configlib_with_defaults() {
        let cfglib = ConfigLib::builder()
            .set_default("server.address", DEFAULT_SERVER_ADDRESS).unwrap()
            .set_default("server.port", DEFAULT_SERVER_PORT).unwrap()
            .set_default("server.tls_enabled", DEFAULT_TLS_ENABLED).unwrap()
            .build().unwrap();

        let ledger_cfg: LedgerConfig = cfglib.try_deserialize().expect("should deserialize");
        assert_eq!(ledger_cfg.server.address, DEFAULT_SERVER_ADDRESS);
        assert_eq!(ledger_cfg.server.port, DEFAULT_SERVER_PORT);
        assert_eq!(ledger_cfg.server.tls_enabled, DEFAULT_TLS_ENABLED);
    }
    use temp_env::with_var;

    #[test]
    fn environment_variable_overrides_take_precedence() {
        // Use temp_env::with_var to set env var for the closure duration
    with_var("LEDGER_BACKEND_SERVER_ADDRESS", Some("0.0.0.0"), || {
            let cfglib = ConfigLib::builder()
                // Provide the same defaults the real loader uses so deserialization succeeds
                .set_default("server.address", DEFAULT_SERVER_ADDRESS).unwrap()
                .set_default("server.port", DEFAULT_SERVER_PORT).unwrap()
                .set_default("server.tls_enabled", DEFAULT_TLS_ENABLED).unwrap()
                .add_source(config::Environment::with_prefix(ENV_SUFFIX).prefix_separator("_").separator("_"))
                .build().unwrap();

            let ledger_cfg: LedgerConfig = cfglib.try_deserialize().expect("should deserialize from env");
            assert_eq!(ledger_cfg.server.address, "0.0.0.0");
        });
    }

    #[test]
    fn server_config_default_is_valid_socket() {
        let s = ServerConfig::default();
        let combined = format!("{}:{}", s.address, s.port);
        assert!(combined.parse::<SocketAddr>().is_ok(), "default server address must parse to SocketAddr");
    }
}
// ...existing code...