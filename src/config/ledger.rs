//! Configuration loader and domain types for the Personal Ledger backend.
//!
//! This module defines `LedgerConfig` (the top-level configuration type)
//! and provides the `parse()` method for loading configuration from defaults,
//! optional config files, and environment variables.
//!
//! Configuration is merged from three sources (in increasing precedence):
//! defaults, an optional INI file under `config/`, and environment variables
//! prefixed with `LEDGER_BACKEND_`.
//!
//! The `ServerConfig` and `ConfigError` types are re-exported by the parent
//! [`super::mod`] module for convenience.

use config::{Config, Environment, File, FileFormat};

/// Environment variable prefix used for overriding configuration values.
///
/// Environment variables are expected in the form `LEDGER_BACKEND_<SECTION>_<KEY>`.
/// For example, to override `server.address` set `LEDGER_BACKEND_SERVER_ADDRESS`.
pub const ENV_SUFFIX: &str = "LEDGER_BACKEND";

/// Top-level application configuration.
///
/// `LedgerConfig` is deserialised from a combination of defaults, an optional
/// INI configuration file and environment variables. The structure currently
/// contains the `server` section (network bind, TLS, and database settings).
#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct LedgerConfig {
    /// Server configuration settings.
    pub server: super::ServerConfig,
}

impl LedgerConfig {
    /// Load the application configuration.
    ///
    /// Sources are applied in order of increasing precedence (later sources
    /// override earlier ones):
    ///
    /// 1. Programmatic defaults (lowest priority)
    /// 2. Optional INI file at `config/<CONFIG_FILE_NAME>.conf` (if present)
    /// 3. Environment variables prefixed with `LEDGER_BACKEND` (highest priority)
    ///
    /// The config file is looked for in the current working directory under
    /// a `config/` directory (for example `config/ledger-backend.conf`). If
    /// the file is absent the loader will continue with defaults and any
    /// provided environment variables.
    ///
    /// Returns a `ConfigError` if there is a problem reading/parsing any of
    /// the sources or deserialising into `LedgerConfig`.
    pub fn parse() -> super::ConfigResult<LedgerConfig> {
        // Get the directory that the binary is being run from
        let binary_path = std::env::current_dir()
            .map_err(|e| super::ConfigError::Validation(format!("Failed to determine current directory: {}", e)))?;

        // Set the configuration directory for the app
        let config_directory = binary_path.join("config");

        // Set the configuration file name to be the package name with .conf extension
        let config_filename = format!("{}.conf", super::server::CONFIG_FILE_NAME);

        // Set the default config file path
        let config_file_path = config_directory.join(config_filename);

        // Start with defaults (lowest priority)
        let mut builder = Config::builder()
            .set_default("server.address", super::server::DEFAULT_SERVER_ADDRESS)?
            .set_default("server.port", super::server::DEFAULT_SERVER_PORT)?
            .set_default("server.data_dir", super::server::DEFAULT_DATA_DIR)?
            .set_default("server.tls_enabled", super::server::DEFAULT_TLS_ENABLED)?;

        // If the config file exists, load it (overrides defaults). If not, warn and continue with defaults
        if config_file_path.exists() {
            // The repository contains `config/ledger-backend.conf` as an INI-style
            // file. Explicitly load the file as INI so the parser accepts `.conf`.
            builder = builder
                .add_source(File::from(config_file_path).format(FileFormat::Ini));
        } else {
            tracing::warn!(
                "Config file '{}' not found; using defaults and environment variables if set",
                config_file_path.display()
            );
        }

        // Finally add environment variables (highest priority)
        builder = builder.add_source(
            Environment::with_prefix(ENV_SUFFIX)
                .prefix_separator("_")
                .separator("_"),
        );

        let config = builder.build()?;

        // Deserialize the generic `config::Config` into our `LedgerConfig` domain type.
        let ledger_config: LedgerConfig = config.try_deserialize()?;

        // Validate the loaded configuration
        // ledger_config.validate()?;

        // Only print configuration details if log level is DEBUG
        if matches!(ledger_config.server.log_level, Some(crate::telemetry::LogLevel::DEBUG)) {
            println!(
                "\n---------------------- [ CONFIGURATION ] ---------------------- \n{:#?} \n---------------------------------------------------------------",
                ledger_config
            );
        }

        Ok(ledger_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::Config as ConfigLib;
    #[test]
    fn environment_variable_overrides_take_precedence() {
        // Use temp_env::with_var to set env var for the closure duration
        temp_env::with_var("LEDGER_BACKEND_SERVER_ADDRESS", Some("0.0.0.0"), || {
            // Use the public LedgerConfig defaults so this test only depends on the
            // LedgerConfig public API rather than internal server constants.
            let defaults = LedgerConfig::default();

            let cfglib = ConfigLib::builder()
                // Provide the same defaults the real loader uses so deserialization succeeds
                .set_default("server.address", defaults.server.address.clone())
                .unwrap()
                .set_default("server.port", defaults.server.port)
                .unwrap()
                .set_default("server.data_dir", defaults.server.data_dir.as_ref().map(|p| p.to_string_lossy().to_string()))
                .unwrap()
                .set_default("server.tls_enabled", defaults.server.tls_enabled)
                .unwrap()
                .add_source(
                    config::Environment::with_prefix(ENV_SUFFIX)
                        .prefix_separator("_")
                        .separator("_"),
                )
                .build()
                .unwrap();

            let ledger_cfg: LedgerConfig = cfglib
                .try_deserialize()
                .expect("should deserialize from env");
            assert_eq!(ledger_cfg.server.address, "0.0.0.0");
        });
    }

}