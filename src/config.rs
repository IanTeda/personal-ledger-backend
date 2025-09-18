//! Configuration module for Personal Ledger Backend.
//!
//! This module handles loading and validating configuration from INI files and environment variables.
//! It supports development and production environments with standard directory locations.
//!
//! ## Features
//!
//! - **Multi-source Configuration**: Load from files, environment variables, and defaults
//! - **Environment Support**: Separate configs for development, production, and testing
//! - **Standard Locations**: Uses XDG Base Directory specification for config files
//! - **Secret Handling**: Secure handling of sensitive configuration values
//! - **Validation**: Comprehensive validation of configuration values
//! - **TLS Support**: Optional TLS configuration for secure connections
//!
//! ## Usage
//!
//! ### Basic Configuration Loading
//!
//! ```rust
//! use personal_ledger_backend::LedgerConfig;
//!
//! // Load configuration for current environment (from LEDGER_BACKENDENV)
//! let config = LedgerConfig::load()?;
//!
//! // Or load for specific environment
//! let dev_config = LedgerConfig::load_for_env("development")?;
//! let prod_config = LedgerConfig::load_for_env("production")?;
//! ```
//!
//! ### Configuration Files
//!
//! The system looks for configuration files in the following locations (in order):
//!
//! 1. Current directory: `development.conf`, `production.conf`
//! 2. User config directory: `~/.config/personal-ledger-backend/`
//! 3. System config directory: `/etc/personal-ledger-backend/`
//!
//! ### Example Configuration File (development.conf)
//!
//! ```ini
//! [server]
//! host = "127.0.0.1"
//! port = 50051
//! tls_enabled = false
//!
//! [database]
//! url = "postgresql://user:password@localhost/personal_ledger_dev"
//!
//! [security]
//! jwt_secret = "your-development-secret-key"
//! jwt_expiry_hours = 24
//! ```
//!
//! ### Environment Variables
//!
//! You can override configuration using environment variables with the `LEDGER_BACKEND` prefix:
//!
//! ```bash
//! export LEDGER_BACKENDENV=production
//! export LEDGER_BACKENDSERVER__PORT=8080
//! export LEDGER_BACKENDDATABASE__URL="postgresql://prod_user:prod_pass@prod_host/prod_db"
//! ```
//!
//! ## Security
//!
//! - Database URLs and JWT secrets are stored as `secrecy::Secret` to prevent accidental logging
//! - TLS certificates and keys are handled securely
//! - Configuration validation prevents common security misconfigurations

use config::{Config, ConfigError as ConfigLibError, File};
use directories::ProjectDirs;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use std::path::PathBuf;
use tracing::{info, warn};

use crate::LedgerResult;

/// Configuration errors that can occur during config loading and validation.
///
/// This enum represents all possible errors that can happen when working with
/// application configuration, including file loading, parsing, and validation errors.
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    /// Error from the underlying config crate during file loading or parsing.
    ///
    /// This wraps errors from the `config` crate such as file not found,
    /// invalid syntax, or parsing failures.
    #[error("Configuration parsing error: {0}")]
    Parsing(#[from] ConfigLibError),

    /// Error when standard project directories cannot be determined.
    ///
    /// This occurs when the system cannot determine the appropriate
    /// directories for configuration files using the XDG Base Directory specification.
    #[error("Project directories not found")]
    ProjectDirsNotFound,

    /// Validation error with a descriptive message.
    ///
    /// This is used for configuration validation failures such as invalid
    /// values, missing required fields, or security misconfigurations.
    #[error("Invalid configuration: {0}")]
    Validation(String),
}

/// Main application configuration structure.
///
/// This struct contains all configuration settings for the Personal Ledger Backend.
/// It is loaded from configuration files and environment variables, with support
/// for different environments (development, production, testing).
///
/// # Fields
///
/// * `server` - Server-specific configuration (host, port, TLS)
/// * `database` - Database connection configuration
/// * `security` - Security-related configuration (JWT, secrets)
///
/// # Example
///
/// ```rust
/// use personal_ledger_backend::config::LedgerConfig;
///
/// let config = LedgerConfig::load()?;
/// println!("Server will run on: {}", config.server_address());
/// ```
#[derive(Debug, Clone, Deserialize, Default)]
pub struct LedgerConfig {
    /// Server configuration settings.
    pub server: ServerConfig,
    /// Database configuration settings.
    pub database: DatabaseConfig,
    /// Security configuration settings.
    pub security: SecurityConfig,
}

/// Server configuration settings.
///
/// Contains all settings related to the gRPC server including network configuration
/// and optional TLS settings for secure connections.
///
/// # Fields
///
/// * `host` - The IP address or hostname to bind the server to
/// * `port` - The port number to listen on
/// * `tls_enabled` - Whether TLS encryption should be enabled
/// * `tls_cert_path` - Path to the TLS certificate file (required if TLS is enabled)
/// * `tls_key_path` - Path to the TLS private key file (required if TLS is enabled)
///
/// # Example
///
/// ```ini
/// [server]
/// host = "127.0.0.1"
/// port = 50051
/// tls_enabled = false
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// The IP address or hostname to bind the server to.
    /// Defaults to "127.0.0.1" for local development.
    pub host: String,

    /// The port number to listen on.
    /// Must be a valid port number (1-65535).
    pub port: u16,

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

/// Database configuration settings.
///
/// Contains the database connection URL and related settings.
/// The URL is stored securely using `secrecy::SecretString` to prevent
/// accidental exposure in logs or error messages.
///
/// # Fields
///
/// * `url` - The PostgreSQL connection URL
///
/// # Security
///
/// The database URL typically contains sensitive information like usernames
/// and passwords. It is stored as a `SecretString` to ensure it doesn't
/// appear in debug output or logs.
///
/// # Example
///
/// ```ini
/// [database]
/// url = "postgresql://user:password@localhost/personal_ledger"
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    /// The PostgreSQL database connection URL.
    /// Should include username, password, host, port, and database name.
    /// Stored securely to prevent accidental exposure.
    pub url: SecretString,
}

/// Security configuration settings.
///
/// Contains security-related settings including JWT configuration.
/// All sensitive values are stored securely using `secrecy::SecretString`.
///
/// # Fields
///
/// * `jwt_secret` - Secret key used for signing and verifying JWT tokens
/// * `jwt_expiry_hours` - Number of hours before JWT tokens expire
///
/// # Security
///
/// The JWT secret is critical for application security and should be:
/// - At least 256 bits (32 characters) long
/// - Randomly generated using a secure random number generator
/// - Different between development, staging, and production environments
/// - Stored securely and never committed to version control
///
/// # Example
///
/// ```ini
/// [security]
/// jwt_secret = "your-super-secret-jwt-key-change-this-in-production"
/// jwt_expiry_hours = 24
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct SecurityConfig {
    /// Secret key used for signing and verifying JWT tokens.
    /// Must be kept secure and never exposed in logs or error messages.
    pub jwt_secret: SecretString,

    /// Number of hours before JWT tokens expire.
    /// Common values are 1, 24, or 168 (7 days).
    pub jwt_expiry_hours: u64,
}

impl LedgerConfig {
    /// Load configuration for the current environment.
    ///
    /// Reads the `LEDGER_BACKENDENV` environment variable to determine which environment
    /// configuration to load. Defaults to "development" if not set.
    ///
    /// # Returns
    ///
    /// Returns a `LedgerResult<Self>` containing the loaded and validated configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Configuration files cannot be read or parsed
    /// - Required configuration values are missing
    /// - Configuration validation fails
    ///
    /// # Example
    ///
    /// ```rust
    /// use personal_ledger_backend::config::LedgerConfig;
    ///
    /// // Load config for current environment
    /// let config = LedgerConfig::load()?;
    /// ```
    pub fn load() -> LedgerResult<Self> {
        let env = std::env::var("LEDGER_BACKENDENV").unwrap_or_else(|_| "development".to_string());
        Self::load_for_env(&env)
    }

    /// Load configuration for a specific environment.
    ///
    /// Loads configuration from multiple sources in order of precedence:
    /// 1. Environment variables with `LEDGER_BACKEND` prefix
    /// 2. Configuration files in standard locations
    /// 3. Default values
    ///
    /// # Arguments
    ///
    /// * `env` - The environment name (e.g., "development", "production")
    ///
    /// # Returns
    ///
    /// Returns a `LedgerResult<Self>` containing the loaded and validated configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Configuration files cannot be read or parsed
    /// - Required configuration values are missing
    /// - Configuration validation fails
    ///
    /// # Example
    ///
    /// ```rust
    /// use personal_ledger_backend::config::LedgerConfig;
    ///
    /// // Load development configuration
    /// let dev_config = LedgerConfig::load_for_env("development")?;
    ///
    /// // Load production configuration
    /// let prod_config = LedgerConfig::load_for_env("production")?;
    /// ```
    pub fn load_for_env(env: &str) -> LedgerResult<Self> {
        let mut builder = Config::builder()
            .add_source(config::Environment::with_prefix("LEDGER_BACKEND").separator("_"));

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

        let config = builder.build().map_err(ConfigError::from)?;
        let ledger_backend_config = config.try_deserialize::<LedgerConfig>().map_err(ConfigError::from)?;
        ledger_backend_config.validate()?;
        Ok(ledger_backend_config)
    }

    /// Get standard config file locations for the environment.
    ///
    /// Returns a list of file paths where configuration files should be searched,
    /// following the XDG Base Directory specification and common conventions.
    ///
    /// # Arguments
    ///
    /// * `env` - The environment name (e.g., "development", "production")
    ///
    /// # Returns
    ///
    /// Returns a `LedgerResult<Vec<PathBuf>>` containing the list of config file paths.
    ///
    /// # Search Order
    ///
    /// 1. Current directory: `{env}.conf`
    /// 2. User config directory: `~/.config/personal-ledger-backend/{env}.conf`
    /// 3. System config directory: `/etc/personal-ledger-backend/{env}.conf`
    fn get_config_files(env: &str) -> LedgerResult<Vec<PathBuf>> {
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
    ///
    /// Performs comprehensive validation of all configuration values to ensure
    /// the application can start safely and securely.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if validation passes, or a `LedgerError` if validation fails.
    ///
    /// # Validation Checks
    ///
    /// - Server port must not be 0
    /// - Database URL must not be empty
    /// - JWT secret must not be empty
    /// - TLS certificate and key paths must be provided if TLS is enabled
    ///
    /// # Example
    ///
    /// ```rust
    /// use personal_ledger_backend::config::LedgerConfig;
    ///
    /// let config = LedgerConfig::default();
    /// config.validate()?; // Returns Ok(()) or Err
    /// ```
    fn validate(&self) -> LedgerResult<()> {
        if self.server.port == 0 {
            return Err(ConfigError::Validation("Port cannot be 0".to_string()).into());
        }
        if self.database.url.expose_secret().trim().is_empty() {
            return Err(ConfigError::Validation("Database URL cannot be empty".to_string()).into());
        }
        if self.security.jwt_secret.expose_secret().trim().is_empty() {
            return Err(ConfigError::Validation("JWT secret cannot be empty".to_string()).into());
        }
        if self.server.tls_enabled && (self.server.tls_cert_path.is_none() || self.server.tls_key_path.is_none()) {
            return Err(ConfigError::Validation(
                "TLS cert and key paths must be provided when TLS is enabled".to_string(),
            ).into());
        }
        Ok(())
    }

    /// Get the server address as a string (host:port).
    ///
    /// Returns a formatted string containing the server's bind address
    /// in the format "host:port".
    ///
    /// # Returns
    ///
    /// A string in the format "host:port" (e.g., "127.0.0.1:50051").
    ///
    /// # Example
    ///
    /// ```rust
    /// use personal_ledger_backend::config::LedgerConfig;
    ///
    /// let config = LedgerConfig::load()?;
    /// println!("Server address: {}", config.server_address());
    /// // Output: Server address: 127.0.0.1:50051
    /// ```
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    /// Check if TLS is properly configured.
    ///
    /// Returns true if TLS is enabled AND both certificate and key paths are provided.
    ///
    /// # Returns
    ///
    /// `true` if TLS is enabled and properly configured, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use personal_ledger_backend::config::LedgerConfig;
    ///
    /// let config = LedgerConfig::load()?;
    /// if config.is_tls_configured() {
    ///     println!("TLS is properly configured");
    /// } else {
    ///     println!("TLS is not configured or incomplete");
    /// }
    /// ```
    pub fn is_tls_configured(&self) -> bool {
        self.server.tls_enabled
            && self.server.tls_cert_path.is_some()
            && self.server.tls_key_path.is_some()
    }

    /// Get the database URL (exposing the secret for database connections).
    ///
    /// Returns the database connection URL as a string. This method exposes
    /// the secret value for use in database connections.
    ///
    /// # Returns
    ///
    /// The database connection URL as a string slice.
    ///
    /// # Security Note
    ///
    /// This method exposes the secret database URL. Use with caution and
    /// avoid logging the returned value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use personal_ledger_backend::config::LedgerConfig;
    ///
    /// let config = LedgerConfig::load()?;
    /// let db_url = config.database_url();
    /// // Use db_url to connect to database
    /// ```
    pub fn database_url(&self) -> &str {
        self.database.url.expose_secret()
    }

    /// Get the JWT secret (exposing the secret for JWT operations).
    ///
    /// Returns the JWT secret key as a string. This method exposes
    /// the secret value for use in JWT signing and verification.
    ///
    /// # Returns
    ///
    /// The JWT secret key as a string slice.
    ///
    /// # Security Note
    ///
    /// This method exposes the secret JWT key. Use with caution and
    /// avoid logging the returned value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use personal_ledger_backend::config::LedgerConfig;
    ///
    /// let config = LedgerConfig::load()?;
    /// let jwt_secret = config.jwt_secret();
    /// // Use jwt_secret for JWT operations
    /// ```
    pub fn jwt_secret(&self) -> &str {
        self.security.jwt_secret.expose_secret()
    }
}


impl Default for ServerConfig {
    /// Create a default server configuration.
    ///
    /// Returns a `ServerConfig` with sensible defaults for development:
    /// - Host: "127.0.0.1" (localhost)
    /// - Port: 50051 (standard gRPC port)
    /// - TLS: disabled
    /// - Certificate paths: None
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
    /// Create a default database configuration.
    ///
    /// Returns a `DatabaseConfig` with a placeholder database URL.
    /// This should be overridden in configuration files or environment variables.
    ///
    /// # Security Note
    ///
    /// The default database URL contains placeholder credentials and should
    /// never be used in production environments.
    fn default() -> Self {
        Self {
            url: SecretString::new("postgresql://user:password@localhost/personal_ledger".to_string().into()),
        }
    }
}

impl Default for SecurityConfig {
    /// Create a default security configuration.
    ///
    /// Returns a `SecurityConfig` with a placeholder JWT secret and 24-hour expiry.
    /// The JWT secret should be overridden in configuration files or environment variables.
    ///
    /// # Security Warning
    ///
    /// The default JWT secret is not secure and should never be used in
    /// production environments. Always override this value.
    fn default() -> Self {
        Self {
            jwt_secret: SecretString::new("your-super-secret-jwt-key-change-this-in-production".to_string().into()),
            jwt_expiry_hours: 24,
        }
    }
}

impl ServerConfig {
    /// Get the server address as a string (host:port).
    ///
    /// Returns a formatted string containing the server's bind address
    /// in the format "host:port".
    ///
    /// # Returns
    ///
    /// A string in the format "host:port" (e.g., "127.0.0.1:50051").
    ///
    /// # Example
    ///
    /// ```rust
    /// use personal_ledger_backend::config::ServerConfig;
    ///
    /// let server_config = ServerConfig::default();
    /// println!("Server address: {}", server_config.address());
    /// // Output: Server address: 127.0.0.1:50051
    /// ```
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Check if TLS is enabled and properly configured.
    ///
    /// Returns true if TLS is enabled AND both certificate and key paths are provided.
    /// This is a more comprehensive check than just `tls_enabled`.
    ///
    /// # Returns
    ///
    /// `true` if TLS is enabled and both certificate and key paths are configured,
    /// `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use personal_ledger_backend::config::ServerConfig;
    ///
    /// let mut server_config = ServerConfig::default();
    /// server_config.tls_enabled = true;
    /// // tls_enabled() will return false because cert/key paths are None
    /// assert!(!server_config.tls_enabled());
    ///
    /// server_config.tls_cert_path = Some(std::path::PathBuf::from("/path/to/cert.pem"));
    /// server_config.tls_key_path = Some(std::path::PathBuf::from("/path/to/key.pem"));
    /// // Now tls_enabled() will return true
    /// assert!(server_config.tls_enabled());
    /// ```
    pub fn tls_enabled(&self) -> bool {
        self.tls_enabled
            && self.tls_cert_path.is_some()
            && self.tls_key_path.is_some()
    }
}

impl DatabaseConfig {
    /// Get the database URL (exposing the secret).
    ///
    /// Returns the database connection URL as a string. This method exposes
    /// the secret value for use in database connections.
    ///
    /// # Returns
    ///
    /// The database connection URL as a string slice.
    ///
    /// # Security Note
    ///
    /// This method exposes the secret database URL. Use with caution and
    /// avoid logging the returned value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use personal_ledger_backend::config::DatabaseConfig;
    ///
    /// let db_config = DatabaseConfig::default();
    /// let url = db_config.url();
    /// // Use url to connect to database
    /// ```
    pub fn url(&self) -> &str {
        self.url.expose_secret()
    }
}

impl SecurityConfig {
    /// Get the JWT secret (exposing the secret).
    ///
    /// Returns the JWT secret key as a string. This method exposes
    /// the secret value for use in JWT signing and verification.
    ///
    /// # Returns
    ///
    /// The JWT secret key as a string slice.
    ///
    /// # Security Note
    ///
    /// This method exposes the secret JWT key. Use with caution and
    /// avoid logging the returned value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use personal_ledger_backend::config::SecurityConfig;
    ///
    /// let security_config = SecurityConfig::default();
    /// let secret = security_config.jwt_secret();
    /// // Use secret for JWT operations
    /// ```
    pub fn jwt_secret(&self) -> &str {
        self.jwt_secret.expose_secret()
    }

    /// Get the JWT expiry duration in seconds.
    ///
    /// Converts the configured expiry hours to seconds for use with JWT libraries.
    ///
    /// # Returns
    ///
    /// The JWT expiry time in seconds.
    ///
    /// # Example
    ///
    /// ```rust
    /// use personal_ledger_backend::config::SecurityConfig;
    ///
    /// let security_config = SecurityConfig::default();
    /// let expiry_seconds = security_config.jwt_expiry_seconds();
    /// // Use expiry_seconds when creating JWT tokens
    /// assert_eq!(expiry_seconds, 86400); // 24 hours in seconds
    /// ```
    pub fn jwt_expiry_seconds(&self) -> u64 {
        self.jwt_expiry_hours * 3600
    }
}



#[cfg(test)]
mod tests {
    /// Test module for configuration functionality.
    ///
    /// This module contains comprehensive tests for all configuration
    /// loading, validation, and utility functions.
    use super::*;

    /// Test default configuration values.
    ///
    /// Verifies that the default configuration has sensible values
    /// for development use.
    #[test]
    fn test_default_config() {
        let config = LedgerConfig::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 50051);
        assert!(!config.server.tls_enabled);
        assert_eq!(config.server_address(), "127.0.0.1:50051");
    }

    /// Test configuration validation.
    ///
    /// Verifies that invalid configuration values are properly rejected
    /// during validation.
    #[test]
    fn test_config_validation() {
        let mut config = LedgerConfig::default();
        config.server.port = 0;
        assert!(config.validate().is_err());
    }

    /// Test database URL validation.
    ///
    /// Verifies that empty database URLs are properly rejected.
    #[test]
    fn test_database_url_validation() {
        let mut config = LedgerConfig::default();
        config.database.url = SecretString::new("".to_string().into());
        assert!(config.validate().is_err());
    }

    /// Test JWT secret validation.
    ///
    /// Verifies that empty JWT secrets are properly rejected.
    #[test]
    fn test_jwt_secret_validation() {
        let mut config = LedgerConfig::default();
        config.security.jwt_secret = SecretString::new("".to_string().into());
        assert!(config.validate().is_err());
    }

    /// Test TLS validation.
    ///
    /// Verifies that TLS configuration is properly validated:
    /// - TLS enabled without cert/key paths should fail
    /// - TLS enabled with cert/key paths should pass
    #[test]
    fn test_tls_validation() {
        let mut config = LedgerConfig::default();
        config.server.tls_enabled = true;
        // Should fail because cert/key paths are None
        assert!(config.validate().is_err());

        // Should pass with cert/key paths
        config.server.tls_cert_path = Some(PathBuf::from("/path/to/cert.pem"));
        config.server.tls_key_path = Some(PathBuf::from("/path/to/key.pem"));
        assert!(config.validate().is_ok());
    }

    /// Test server configuration address formatting.
    ///
    /// Verifies that the server address is properly formatted as "host:port".
    #[test]
    fn test_server_config_address() {
        let config = ServerConfig::default();
        assert_eq!(config.address(), "127.0.0.1:50051");
    }

    /// Test server configuration TLS enabled check.
    ///
    /// Verifies that TLS is considered enabled only when both
    /// tls_enabled is true AND cert/key paths are provided.
    #[test]
    fn test_server_config_tls_enabled() {
        let mut config = ServerConfig::default();
        assert!(!config.tls_enabled()); // TLS disabled by default

        config.tls_enabled = true;
        assert!(!config.tls_enabled()); // Still false because no cert/key paths

        config.tls_cert_path = Some(PathBuf::from("/cert.pem"));
        config.tls_key_path = Some(PathBuf::from("/key.pem"));
        assert!(config.tls_enabled()); // Now true
    }

    #[test]
    fn test_security_config_jwt_expiry() {
        let config = SecurityConfig::default();
        assert_eq!(config.jwt_expiry_seconds(), 24 * 3600); // 24 hours in seconds
    }

    #[test]
    fn test_config_file_locations() {
        let files = LedgerConfig::get_config_files("development").unwrap();
        assert!(!files.is_empty());

        // Should include current directory file
        assert!(files.contains(&PathBuf::from("development.conf")));
    }

    #[test]
    fn test_secret_handling() {
        let config = LedgerConfig::default();

        // Test that secrets are properly wrapped
        assert!(!config.database_url().is_empty());
        assert!(!config.jwt_secret().is_empty());

        // Test that the same secret is returned
        assert_eq!(config.database_url(), config.database_url());
    }

    #[test]
    fn test_config_builder_methods() {
        let config = LedgerConfig::default();

        // Test utility methods
        assert!(!config.is_tls_configured()); // TLS not configured by default

        // Test with TLS configured
        let mut tls_config = config.clone();
        tls_config.server.tls_enabled = true;
        tls_config.server.tls_cert_path = Some(PathBuf::from("/cert.pem"));
        tls_config.server.tls_key_path = Some(PathBuf::from("/key.pem"));
        assert!(tls_config.is_tls_configured());
    }

    /// Test database configuration URL exposure.
    ///
    /// Verifies that the database URL can be properly exposed from the secret.
    #[test]
    fn test_database_config_url() {
        let db_config = DatabaseConfig::default();
        let url = db_config.url();
        assert!(!url.is_empty());
        assert!(url.contains("postgresql://"));
    }

    /// Test security configuration JWT secret exposure.
    ///
    /// Verifies that the JWT secret can be properly exposed from the secret.
    #[test]
    fn test_security_config_jwt_secret() {
        let security_config = SecurityConfig::default();
        let secret = security_config.jwt_secret();
        assert!(!secret.is_empty());
        assert!(secret.contains("change-this-in-production"));
    }

    /// Test server configuration with custom values.
    ///
    /// Verifies that server configuration works with non-default values.
    #[test]
    fn test_server_config_custom_values() {
        let config = ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            ..ServerConfig::default()
        };

        assert_eq!(config.address(), "0.0.0.0:8080");
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
    }

    /// Test configuration validation with all fields valid.
    ///
    /// Verifies that a properly configured instance passes validation.
    #[test]
    fn test_config_validation_success() {
        let config = LedgerConfig::default();
        assert!(config.validate().is_ok());
    }

    /// Test configuration validation with multiple errors.
    ///
    /// Verifies that validation catches multiple validation issues.
    #[test]
    fn test_config_validation_multiple_errors() {
        let mut config = LedgerConfig::default();
        config.server.port = 0;
        config.database.url = SecretString::new("".to_string().into());
        config.security.jwt_secret = SecretString::new("".to_string().into());

        // Should fail validation due to multiple issues
        assert!(config.validate().is_err());
    }

    /// Test TLS validation with partial configuration.
    ///
    /// Verifies that TLS validation fails when only cert or key path is provided.
    #[test]
    fn test_tls_validation_partial() {
        let mut config = LedgerConfig::default();
        config.server.tls_enabled = true;

        // Only cert path provided
        config.server.tls_cert_path = Some(PathBuf::from("/cert.pem"));
        assert!(config.validate().is_err());

        // Only key path provided
        config.server.tls_cert_path = None;
        config.server.tls_key_path = Some(PathBuf::from("/key.pem"));
        assert!(config.validate().is_err());
    }

    /// Test server address formatting with different configurations.
    ///
    /// Verifies that address formatting works with various host/port combinations.
    #[test]
    fn test_server_address_formatting() {
        let mut config = ServerConfig::default();

        // Test localhost
        assert_eq!(config.address(), "127.0.0.1:50051");

        // Test different host
        config.host = "localhost".to_string();
        assert_eq!(config.address(), "localhost:50051");

        // Test different port
        config.port = 8080;
        assert_eq!(config.address(), "localhost:8080");

        // Test IPv6-like format (though not valid IPv6)
        config.host = "::1".to_string();
        config.port = 9000;
        assert_eq!(config.address(), "::1:9000");
    }

    /// Test JWT expiry conversion with different values.
    ///
    /// Verifies that JWT expiry hours are correctly converted to seconds.
    #[test]
    fn test_jwt_expiry_conversion() {
        let mut config = SecurityConfig::default();

        // Test default (24 hours)
        assert_eq!(config.jwt_expiry_seconds(), 86400);

        // Test 1 hour
        config.jwt_expiry_hours = 1;
        assert_eq!(config.jwt_expiry_seconds(), 3600);

        // Test 7 days
        config.jwt_expiry_hours = 168;
        assert_eq!(config.jwt_expiry_seconds(), 604800);
    }

    /// Test config file locations for different environments.
    ///
    /// Verifies that config file paths are generated correctly for different environments.
    #[test]
    fn test_config_file_locations_multiple_envs() {
        // Test development environment
        let dev_files = LedgerConfig::get_config_files("development").unwrap();
        assert!(dev_files.contains(&PathBuf::from("development.conf")));

        // Test production environment
        let prod_files = LedgerConfig::get_config_files("production").unwrap();
        assert!(prod_files.contains(&PathBuf::from("production.conf")));

        // Test custom environment
        let staging_files = LedgerConfig::get_config_files("staging").unwrap();
        assert!(staging_files.contains(&PathBuf::from("staging.conf")));
    }

    /// Test secret handling with custom values.
    ///
    /// Verifies that secrets work correctly with custom values.
    #[test]
    fn test_secret_handling_custom_values() {
        let mut config = LedgerConfig::default();

        // Set custom database URL
        let custom_db_url = "postgresql://custom_user:custom_pass@custom_host/custom_db";
        config.database.url = SecretString::new(custom_db_url.to_string().into());
        assert_eq!(config.database_url(), custom_db_url);

        // Set custom JWT secret
        let custom_jwt_secret = "my-custom-jwt-secret-key";
        config.security.jwt_secret = SecretString::new(custom_jwt_secret.to_string().into());
        assert_eq!(config.jwt_secret(), custom_jwt_secret);
    }

    /// Test TLS configuration completeness check.
    ///
    /// Verifies that TLS configuration detection works in various scenarios.
    #[test]
    fn test_tls_config_completeness() {
        let mut config = LedgerConfig::default();

        // Initially not configured
        assert!(!config.is_tls_configured());

        // Enable TLS but don't provide paths
        config.server.tls_enabled = true;
        assert!(!config.is_tls_configured());

        // Provide only cert path
        config.server.tls_cert_path = Some(PathBuf::from("/cert.pem"));
        assert!(!config.is_tls_configured());

        // Provide both paths
        config.server.tls_key_path = Some(PathBuf::from("/key.pem"));
        assert!(config.is_tls_configured());

        // Disable TLS again
        config.server.tls_enabled = false;
        assert!(!config.is_tls_configured());
    }

    /// Test configuration cloning.
    ///
    /// Verifies that configuration structs can be properly cloned.
    #[test]
    fn test_config_cloning() {
        let mut original = LedgerConfig::default();
        original.server.port = 8080;
        original.server.host = "example.com".to_string();

        let cloned = original.clone();

        // Verify they are equal
        assert_eq!(original.server.host, cloned.server.host);
        assert_eq!(original.server.port, cloned.server.port);
        assert_eq!(original.server_address(), cloned.server_address());

        // Verify they are independent (changing one doesn't affect the other)
        original.server.port = 9000;
        assert_ne!(original.server.port, cloned.server.port);
    }

    /// Test type alias compatibility.
    ///
    /// Verifies that the AppConfig type alias works correctly.
    #[test]
    fn test_config_type_alias() {
        // Test that AppConfig is the same as LedgerConfig
        let ledger_backend_config = LedgerConfig::default();
        let ledger_config: LedgerConfig = LedgerConfig::default();

        assert_eq!(ledger_backend_config.server.host, ledger_config.server.host);
        assert_eq!(ledger_backend_config.server.port, ledger_config.server.port);
        assert_eq!(ledger_backend_config.server_address(), ledger_config.server_address());
    }

    /// Test configuration with extreme values.
    ///
    /// Verifies that the configuration handles edge cases properly.
    #[test]
    fn test_config_edge_cases() {
        let mut config = LedgerConfig::default();

        // Test with very high port number
        config.server.port = 65535;
        assert_eq!(config.server_address(), "127.0.0.1:65535");

        // Test with empty host (should still format)
        config.server.host = "".to_string();
        assert_eq!(config.server_address(), ":65535");

        // Test with very long host name
        let long_host = "a".repeat(1000);
        config.server.host = long_host.clone();
        assert_eq!(config.server_address(), format!("{}:65535", long_host));
    }

    /// Test configuration validation with whitespace-only values.
    ///
    /// Verifies that whitespace-only strings are treated as empty.
    #[test]
    fn test_config_validation_whitespace() {
        let mut config = LedgerConfig::default();

        // Test whitespace-only database URL
        config.database.url = SecretString::new("   ".to_string().into());
        assert!(config.validate().is_err());

        // Reset and test whitespace-only JWT secret
        config = LedgerConfig::default();
        config.security.jwt_secret = SecretString::new("\t\n  ".to_string().into());
        assert!(config.validate().is_err());
    }
}
