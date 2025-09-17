//! Configuration module for Personal Ledger Backend.
//!
//! This module handles loading and validating configuration from INI files and environment variables.
//! It supports development and production environments with standard directory locations.

use config::{Config, ConfigError, File};
use directories::ProjectDirs;
use secrecy::{ExposeSecret, Secret};
use std::path::PathBuf;
use thiserror::Error;
use tracing::{info, warn};

/// Configuration errors.
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration loading error: {0}")]
    Config(#[from] ConfigError),
    #[error("Project directories not found")]
    ProjectDirsNotFound,
    #[error("Invalid configuration: {0}")]
    Validation(String),
}

/// Main application configuration.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub security: SecurityConfig,
}

/// Server configuration.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub tls_enabled: bool,
    pub tls_cert_path: Option<PathBuf>,
    pub tls_key_path: Option<PathBuf>,
}

/// Database configuration.
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: Secret<String>,
}

/// Security configuration.
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub jwt_secret: Secret<String>,
    pub jwt_expiry_hours: u64,
}

impl AppConfig {
    /// Load configuration for the specified environment.
    pub fn load() -> Result<Self, ConfigError> {
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
        Self::load_for_env(&env)
    }

    /// Load configuration for a specific environment.
    pub fn load_for_env(env: &str) -> Result<Self, ConfigError> {
        let mut builder = Config::builder()
            .add_source(config::Environment::with_prefix("APP").separator("_"));

        // Add config files from standard locations
        let config_files = Self::get_config_files(env)?;
        for file in config_files {
            if file.exists() {
                info!("Loading config from: {}", file.display());
                builder = builder.add_source(File::from(file).required(false));
            } else {
                warn!("Config file not found: {}", file.display());
            }
        }

        let config = builder.build()?;
        let app_config = config.try_deserialize::<AppConfig>()?;
        app_config.validate()?;
        Ok(app_config)
    }

    /// Get standard config file locations for the environment.
    fn get_config_files(env: &str) -> Result<Vec<PathBuf>, ConfigError> {
        let mut files = Vec::new();

        // Add current directory
        files.push(PathBuf::from(format!("{}.conf", env)));

        // Add standard directories
        if let Some(proj_dirs) = ProjectDirs::from("com", "PersonalLedger", "Backend") {
            files.push(proj_dirs.config_dir().join(format!("{}.conf", env)));
        }

        // Add system-wide locations
        files.push(PathBuf::from("/etc/personal-ledger-backend").join(format!("{}.conf", env)));

        Ok(files)
    }

    /// Validate the configuration.
    fn validate(&self) -> Result<(), ConfigError> {
        if self.server.port == 0 {
            return Err(ConfigError::Validation("Port cannot be 0".to_string()));
        }
        if self.database.url.expose_secret().is_empty() {
            return Err(ConfigError::Validation("Database URL cannot be empty".to_string()));
        }
        if self.security.jwt_secret.expose_secret().is_empty() {
            return Err(ConfigError::Validation("JWT secret cannot be empty".to_string()));
        }
        if self.server.tls_enabled {
            if self.server.tls_cert_path.is_none() || self.server.tls_key_path.is_none() {
                return Err(ConfigError::Validation(
                    "TLS cert and key paths must be provided when TLS is enabled".to_string(),
                ));
            }
        }
        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 50051,
            tls_enabled: false,
            tls_cert_path: None,
            tls_key_path: None,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: Secret::new("postgresql://user:password@localhost/personal_ledger".to_string()),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            jwt_secret: Secret::new("your-secret-key".to_string()),
            jwt_expiry_hours: 24,
        }
    }
}

// Implement deserialization for config crate
impl<'de> serde::Deserialize<'de> for AppConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct AppConfigHelper {
            server: ServerConfig,
            database: DatabaseConfig,
            security: SecurityConfig,
        }

        let helper = AppConfigHelper::deserialize(deserializer)?;
        Ok(AppConfig {
            server: helper.server,
            database: helper.database,
            security: helper.security,
        })
    }
}

impl<'de> serde::Deserialize<'de> for ServerConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct ServerConfigHelper {
            host: String,
            port: u16,
            tls_enabled: Option<bool>,
            tls_cert_path: Option<String>,
            tls_key_path: Option<String>,
        }

        let helper = ServerConfigHelper::deserialize(deserializer)?;
        Ok(ServerConfig {
            host: helper.host,
            port: helper.port,
            tls_enabled: helper.tls_enabled.unwrap_or(false),
            tls_cert_path: helper.tls_cert_path.map(PathBuf::from),
            tls_key_path: helper.tls_key_path.map(PathBuf::from),
        })
    }
}

impl<'de> serde::Deserialize<'de> for DatabaseConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct DatabaseConfigHelper {
            url: String,
        }

        let helper = DatabaseConfigHelper::deserialize(deserializer)?;
        Ok(DatabaseConfig {
            url: Secret::new(helper.url),
        })
    }
}

impl<'de> serde::Deserialize<'de> for SecurityConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct SecurityConfigHelper {
            jwt_secret: String,
            jwt_expiry_hours: Option<u64>,
        }

        let helper = SecurityConfigHelper::deserialize(deserializer)?;
        Ok(SecurityConfig {
            jwt_secret: Secret::new(helper.jwt_secret),
            jwt_expiry_hours: helper.jwt_expiry_hours.unwrap_or(24),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 50051);
        assert!(!config.server.tls_enabled);
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        config.server.port = 0;
        assert!(config.validate().is_err());
    }
}
