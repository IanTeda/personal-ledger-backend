//! Configuration loader and domain types for the Personal Ledger backend.
//!
//! This module exposes `LedgerConfig` (the top-level configuration type)
//! and re-exports `ServerConfig` plus the `ConfigError` type used for
//! parse-time failures.
//!
//! Configuration is merged from three sources (in increasing precedence):
//! defaults, an optional INI file under `config/`, and environment variables
//! prefixed with `LEDGER_BACKEND`.

use config::Config as ConfigLib;

use crate::config::{database, server};

/// Environment variable prefix used for overriding configuration values.
///
/// Environment variables are expected in the form `LEDGER_BACKEND_<SECTION>_<KEY>`.
/// For example, to override `server.address` set `LEDGER_BACKEND_SERVER_ADDRESS`.
pub const ENV_SUFFIX: &str = "LEDGER_BACKEND";

/// Top-level application configuration.
///
/// `LedgerConfig` is deserialised from a combination of defaults, an optional
/// INI configuration file and environment variables. The structure currently
/// only contains the `server` section (network bind and TLS settings), but the
/// type is intentionally extensible for additional sections such as database
/// configuration in future.
#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct LedgerConfig {
    /// Server configuration settings.
    pub server: server::ServerConfig,

    /// Database configuration settings.
    pub database: database::DatabaseConfig,
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
    pub fn parse() -> Result<LedgerConfig, crate::config::ConfigError> {
        // Get the directory that the binary is being run from
        let binary_path =
            std::env::current_dir().expect("Failed to determine the current directory");

        // Set the configuration directory for the app
        let config_directory = binary_path.join("config");

        // Set the configuration file name to be the package name with .conf extension
        let config_filename = format!("{}.conf", server::CONFIG_FILE_NAME);

        // Set the default config file path
        let config_file_path = config_directory.join(config_filename);

        // Start with defaults (lowest priority)
        let mut builder = ConfigLib::builder()
            .set_default("server.address", server::DEFAULT_SERVER_ADDRESS)?
            .set_default("server.port", server::DEFAULT_SERVER_PORT)?
            .set_default("server.data_dir", server::DEFAULT_DATA_DIR)?
            .set_default("server.tls_enabled", server::DEFAULT_TLS_ENABLED)?
            .set_default("database.kind", database::DEFAULT_DB_ENGINE.to_string())?;

        // If the config file exists, load it (overrides defaults). If not, warn and continue with defaults
        if config_file_path.exists() {
            // The repository contains `config/ledger-backend.conf` as an INI-style
            // file. Explicitly load the file as INI so the parser accepts `.conf`.
            builder = builder
                .add_source(config::File::from(config_file_path).format(config::FileFormat::Ini));
        } else {
            tracing::warn!(
                "Config file '{}' not found; using defaults and environment variables",
                config_file_path.display()
            );
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

        println!(
            "\n---------------------- [ CONFIGURATION ] ---------------------- \n{:#?} \n---------------------------------------------------------------",
            ledger_config
        );

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
                .set_default("database.kind", defaults.database.kind.to_string())
                .unwrap()
                .set_default("database.database", defaults.database.database.clone())
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