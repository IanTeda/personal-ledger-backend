//! Database configuration and connection helpers.
//!
//! This module defines the [`DatabaseConfig`] and [`PostgresConfig`] types, as well as
//! constants and helpers for creating SQLx connection pools for the configured engine.
//!
//! # Supported Engines
//!
//! - [`DbEngine::Sqlite`](crate::domain::DbEngine::Sqlite): Default for local development and tests
//! - [`DbEngine::Postgres`](crate::domain::DbEngine::Postgres): Recommended for production
//!
//! # Defaults
//!
//! Defaults are intentionally conservative (in-memory SQLite) to make tests and local
//! development simple. Production deployments should provide a Postgres configuration via
//! the `postgres` section or the `url` field.
//!
//! # Example
//!
//! ```rust
//! use personal_ledger_backend::config::{DatabaseConfig, PostgresConfig};
//! use personal_ledger_backend::domain::DbEngine;
//! use secrecy::SecretString;
//!
//! // Default (SQLite)
//! let config = DatabaseConfig::default();
//! assert_eq!(config.kind, DbEngine::Sqlite);
//!
//! // Postgres
//! let pg = PostgresConfig {
//!     host: "localhost".to_string(),
//!     port: 5432,
//!     user: "user".to_string(),
//!     password: SecretString::new("pw".into()),
//!     url: None,
//!     ssl_mode: Some("require".to_string()),
//!     max_pool_size: Some(10),
//!     connect_timeout_secs: Some(30),
//! };
//! let config = DatabaseConfig {
//!     kind: DbEngine::Postgres,
//!     database: "ledger".to_string(),
//!     postgres: Some(pg),
//! };
//! ```

// Re-export the secrecy trait used when composing connection URLs.
use secrecy::ExposeSecret;

use crate::config::ConfigError;
use crate::domain;

/// Default database engine used when no configuration is provided.
///
/// Defaults to [`DbEngine::Sqlite`](crate::domain::DbEngine::Sqlite) for ease of
/// local development and testing. Production deployments should configure PostgreSQL
/// via the `postgres` section in configuration files.
pub const DEFAULT_DB_ENGINE: domain::DbEngine = domain::DbEngine::Sqlite;

/// Default database name used when no value is provided by configuration.
///
/// Defaults to `"personal_ledger"` to match the application name. Production
/// deployments should override this with an environment-specific database name
/// (e.g., `personal_ledger_prod`, `personal_ledger_staging`).
pub const DEFAULT_DATABASE: &str = "personal_ledger";

/// Default PostgreSQL configuration (none by default).
///
/// This is `None` by default, requiring explicit configuration for PostgreSQL
/// connections. When using SQLite (the default engine), this field is not used.
pub const DEFAULT_POSTGRES: Option<PostgresConfig> = None;


/// Top-level database configuration for connecting to SQLite or PostgreSQL.
///
/// This structure encapsulates all database connection parameters including the
/// database engine type, database name, and engine-specific configuration options.
/// The configuration can be loaded from TOML files, environment variables, or
/// constructed programmatically.
///
/// # Fields
///
/// - `kind`: The database engine to use ([`DbEngine::Sqlite`](crate::domain::DbEngine::Sqlite)
///   or [`DbEngine::Postgres`](crate::domain::DbEngine::Postgres))
/// - `database`: Name of the database file (SQLite) or database name (PostgreSQL)
/// - `postgres`: PostgreSQL-specific settings, required when `kind` is `Postgres`
///
/// # Example
///
/// ```rust
/// use personal_ledger_backend::config::{DatabaseConfig, PostgresConfig};
/// use personal_ledger_backend::domain::DbEngine;
/// use secrecy::SecretString;
///
/// // SQLite configuration (default)
/// let sqlite_config = DatabaseConfig::default();
/// assert_eq!(sqlite_config.kind, DbEngine::Sqlite);
///
/// // PostgreSQL configuration
/// let postgres_config = DatabaseConfig {
///     kind: DbEngine::Postgres,
///     database: "my_ledger".to_string(),
///     postgres: Some(PostgresConfig {
///         host: "localhost".to_string(),
///         port: 5432,
///         user: "ledger_user".to_string(),
///         password: SecretString::new("secure_password".into()),
///         url: None,
///         ssl_mode: Some("require".to_string()),
///         max_pool_size: Some(10),
///         connect_timeout_secs: Some(30),
///     }),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub struct DatabaseConfig {
    /// The database engine type (SQLite or PostgreSQL).
    ///
    /// Determines which database backend to use and which configuration
    /// parameters are required. Defaults to [`DbEngine::Sqlite`](crate::domain::DbEngine::Sqlite).
    pub kind: domain::DbEngine,

    /// Database name or file path.
    ///
    /// For SQLite: the file path without extension (`.db` is appended automatically)
    /// or `":memory:"` for in-memory databases.
    /// For PostgreSQL: the database name on the PostgreSQL server.
    pub database: String,

    /// PostgreSQL-specific connection configuration.
    ///
    /// Required when `kind` is [`DbEngine::Postgres`](crate::domain::DbEngine::Postgres).
    /// Must be `None` when using SQLite. Contains connection parameters like
    /// host, port, credentials, and connection pool settings.
    pub postgres: Option<PostgresConfig>,
}


impl Default for DatabaseConfig {
    /// Creates a default database configuration using SQLite with in-memory storage.
    ///
    /// This is suitable for testing and local development. The default configuration
    /// uses SQLite with the database name "personal_ledger" (no PostgreSQL config).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use personal_ledger_backend::config::DatabaseConfig;
    /// # use personal_ledger_backend::domain::DbEngine;
    /// let config = DatabaseConfig::default();
    /// assert_eq!(config.kind, DbEngine::Sqlite);
    /// assert_eq!(config.database, "personal_ledger");
    /// assert!(config.postgres.is_none());
    /// ```
    fn default() -> Self {
        Self {
            kind: DEFAULT_DB_ENGINE,
            database: DEFAULT_DATABASE.to_string(),
            postgres: DEFAULT_POSTGRES,
        }
    }
}

/// PostgreSQL connection configuration.
///
/// Contains all parameters required to establish a connection to a PostgreSQL
/// database server. Supports both direct field-based configuration and full
/// connection URL specification.
///
/// # Connection URL Priority
///
/// If the `url` field is provided, it takes precedence over individual fields
/// (`host`, `port`, `user`, `password`). Otherwise, a connection URL is
/// composed from the individual fields.
///
/// # Fields
///
/// - `host`: PostgreSQL server hostname or IP address
/// - `port`: PostgreSQL server port (typically 5432)
/// - `user`: Database user for authentication
/// - `password`: User password (stored securely, not logged)
/// - `url`: Optional complete connection URL (overrides field-based config)
/// - `ssl_mode`: Optional SSL/TLS mode (`disable`, `prefer`, `require`, etc.)
/// - `max_pool_size`: Optional maximum connections in the connection pool
/// - `connect_timeout_secs`: Optional connection timeout in seconds
///
/// # Example
///
/// ```rust
/// use personal_ledger_backend::config::PostgresConfig;
/// use secrecy::SecretString;
///
/// // Field-based configuration
/// let config = PostgresConfig {
///     host: "db.example.com".to_string(),
///     port: 5432,
///     user: "app_user".to_string(),
///     password: SecretString::new("secret123".into()),
///     url: None,
///     ssl_mode: Some("require".to_string()),
///     max_pool_size: Some(20),
///     connect_timeout_secs: Some(30),
/// };
///
/// // URL-based configuration (preferred)
/// let config_with_url = PostgresConfig {
///     host: "".to_string(), // Ignored when url is present
///     port: 0,              // Ignored when url is present
///     user: "".to_string(), // Ignored when url is present
///     password: SecretString::new("".into()), // Ignored when url is present
///     url: Some("postgres://user:pass@db.example.com:5432/mydb?sslmode=require".to_string()),
///     ssl_mode: None,
///     max_pool_size: Some(10),
///     connect_timeout_secs: Some(30),
/// };
/// ```
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PostgresConfig {
    /// PostgreSQL server hostname or IP address.
    pub host: String,

    /// PostgreSQL server TCP port.
    pub port: u16,

    /// Database username for authentication.
    pub user: String,

    /// Database user password.
    pub password: secrecy::SecretString,

    /// Complete PostgreSQL connection URL (optional, takes precedence).
    pub url: Option<String>,

    /// SSL/TLS connection mode.
    pub ssl_mode: Option<String>,

    /// Maximum number of connections in the connection pool.
    pub max_pool_size: Option<u32>,

    /// Connection timeout in seconds.
    pub connect_timeout_secs: Option<u64>,
}

impl PartialEq for PostgresConfig {
    /// Compares two PostgreSQL configurations for equality.
    ///
    /// This implementation compares all fields including the password by exposing
    /// its secret value. Use with caution in security-sensitive contexts.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use personal_ledger_backend::config::PostgresConfig;
    /// # use secrecy::SecretString;
    /// let config1 = PostgresConfig {
    ///     host: "localhost".to_string(),
    ///     port: 5432,
    ///     user: "user".to_string(),
    ///     password: SecretString::new("pass".into()),
    ///     url: None,
    ///     ssl_mode: None,
    ///     max_pool_size: None,
    ///     connect_timeout_secs: None,
    /// };
    ///
    /// let config2 = config1.clone();
    /// assert_eq!(config1, config2);
    /// ```
    fn eq(&self, other: &Self) -> bool {
        self.host == other.host
            && self.port == other.port
            && self.user == other.user
            && self.password.expose_secret() == other.password.expose_secret()
            && self.url == other.url
            && self.ssl_mode == other.ssl_mode
            && self.max_pool_size == other.max_pool_size
            && self.connect_timeout_secs == other.connect_timeout_secs
    }
}

impl DatabaseConfig {
    /// Constructs a database connection URL from the configuration.
    ///
    /// Builds the appropriate connection string based on the configured database
    /// engine (`kind`). For SQLite, creates a file-based or in-memory URL. For
    /// PostgreSQL, either uses the provided full URL or composes one from
    /// individual connection parameters.
    ///
    /// # Returns
    ///
    /// Returns a connection URL string suitable for SQLx connection establishment:
    /// - SQLite: `"sqlite:database_name.db"` or `"sqlite::memory:"`
    /// - PostgreSQL: `"postgres://user:pass@host:port/database?sslmode=mode"`
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::Validation`](crate::config::ConfigError::Validation) if:
    /// - Database engine is PostgreSQL but no `postgres` configuration is provided
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use personal_ledger_backend::config::{DatabaseConfig, PostgresConfig};
    /// # use personal_ledger_backend::domain::DbEngine;
    /// # use secrecy::SecretString;
    /// // SQLite URL
    /// let sqlite_config = DatabaseConfig {
    ///     kind: DbEngine::Sqlite,
    ///     database: "my_ledger".to_string(),
    ///     postgres: None,
    /// };
    /// let url = sqlite_config.connection_url().unwrap();
    /// assert_eq!(url, "sqlite:my_ledger.db");
    ///
    /// // PostgreSQL URL (composed from fields)
    /// let postgres_config = DatabaseConfig {
    ///     kind: DbEngine::Postgres,
    ///     database: "ledger_db".to_string(),
    ///     postgres: Some(PostgresConfig {
    ///         host: "localhost".to_string(),
    ///         port: 5432,
    ///         user: "admin".to_string(),
    ///         password: SecretString::new("secure".into()),
    ///         url: None,
    ///         ssl_mode: Some("require".to_string()),
    ///         max_pool_size: None,
    ///         connect_timeout_secs: None,
    ///     }),
    /// };
    /// let url = postgres_config.connection_url().unwrap();
    /// assert_eq!(url, "postgres://admin:secure@localhost:5432/ledger_db?sslmode=require");
    /// ```
    ///
    /// # Notes
    ///
    /// - For SQLite, the `.db` extension is automatically appended to the database name
    /// - PostgreSQL passwords are exposed from `SecretString` only during URL construction
    /// - SSL mode is only appended to PostgreSQL URLs when `ssl_mode` is `Some`
    /// - Connection pool settings (`max_pool_size`, `connect_timeout_secs`) are not
    ///   currently used but reserved for future SQLx pool configuration
    pub fn connection_url(
        &self,
    ) -> Result<String, ConfigError> {
        let url = match self.kind {
            domain::DbEngine::Sqlite => {
                 format!("sqlite:{}.db", self.database)
            }
            domain::DbEngine::Postgres => {
                // Ensure we have a Postgres config
                let pg = match &self.postgres {
                    Some(cfg) => cfg,
                    None => {
                        return Err(ConfigError::Validation(
                            "postgres configuration missing".to_string(),
                        ));
                    }
                };

                // If a full URL is provided, use it; otherwise compose one from fields
                if let Some(u) = &pg.url {
                    u.clone()
                } else {
                    let mut base = format!(
                        "postgres://{}:{}@{}:{}/{}",
                        pg.user,
                        pg.password.expose_secret(),
                        pg.host,
                        pg.port,
                        self.database
                    );
                    if let Some(ssl) = &pg.ssl_mode {
                        base = format!("{}?sslmode={}", base, ssl);
                    }
                    base
                }
            }
        };
        
        tracing::info!("Constructed database URL for");

        Ok(url)
    }

    #[cfg(test)]
    pub fn test_connection_url(&self) -> Result<String, ConfigError> {
        use uuid::Uuid;

        // Randomly select database engine for testing
        let random_engine = domain::DbEngine::random();

        let url = match random_engine {
            domain::DbEngine::Sqlite => {
                // Use in-memory SQLite for tests to ensure isolation
                "sqlite::memory:".to_string()
            }
            domain::DbEngine::Postgres => {
                // Ensure we have a Postgres config
                let pg = match &self.postgres {
                    Some(cfg) => cfg,
                    None => {
                        return Err(ConfigError::Validation(
                            "postgres configuration missing for random postgres test".to_string(),
                        ));
                    }
                };

                // Use a random database name for test isolation
                let test_database = format!("test_{}", Uuid::now_v7().simple()).replace('-', "_");

                // If a full URL is provided, modify it; otherwise compose one from fields
                if let Some(u) = &pg.url {
                    // Replace the database name in the URL with a test database
                    // Need to handle URLs with query parameters properly
                    if let Some(query_start) = u.find('?') {
                        // URL has query parameters: postgres://host:port/db?query
                        let base_with_host = &u[..query_start]; // postgres://host:port/db
                        if let Some(db_start) = base_with_host.rfind('/') {
                            let base_url = &base_with_host[..=db_start]; // postgres://host:port/
                            let query_part = &u[query_start..]; // ?query
                            format!("{}{}{}", base_url, test_database, query_part)
                        } else {
                            // Fallback: couldn't find database separator
                            format!("{}/{}?{}", base_with_host, test_database, &u[query_start + 1..])
                        }
                    } else {
                        // URL without query parameters: postgres://host:port/db
                        if let Some(db_start) = u.rfind('/') {
                            let base_url = &u[..=db_start];
                            format!("{}{}", base_url, test_database)
                        } else {
                            // Fallback: append test database if no clear database separator
                            format!("{}/{}", u, test_database)
                        }
                    }
                } else {
                    let mut base = format!(
                        "postgres://{}:{}@{}:{}/{}",
                        pg.user,
                        pg.password.expose_secret(),
                        pg.host,
                        pg.port,
                        test_database
                    );
                    if let Some(ssl) = &pg.ssl_mode {
                        base = format!("{}?sslmode={}", base, ssl);
                    }
                    base
                }
            }
        };

        tracing::info!("Constructed test database URL (random engine: {}): {}", random_engine, url);

        Ok(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::SecretString;

    #[test]
    fn test_default_constants() {
        assert_eq!(DEFAULT_DB_ENGINE, domain::DbEngine::Sqlite);
        assert_eq!(DEFAULT_DATABASE, "personal_ledger");
        assert_eq!(DEFAULT_POSTGRES, None);
    }

    #[test]
    fn test_database_config_default() {
        let config = DatabaseConfig::default();
        assert_eq!(config.kind, domain::DbEngine::Sqlite);
        assert_eq!(config.database, "personal_ledger");
        assert!(config.postgres.is_none());
    }

    #[test]
    fn test_postgres_config_partial_eq() {
        let config1 = PostgresConfig {
            host: "localhost".to_string(),
            port: 5432,
            user: "testuser".to_string(),
            password: SecretString::new("testpass".into()),
            url: None,
            ssl_mode: Some("require".to_string()),
            max_pool_size: Some(10),
            connect_timeout_secs: Some(30),
        };

        let config2 = config1.clone();
        assert_eq!(config1, config2);

        let config3 = PostgresConfig {
            host: "different".to_string(),
            ..config1.clone()
        };
        assert_ne!(config1, config3);
    }

    #[test]
    fn test_postgres_config_password_comparison() {
        let config1 = PostgresConfig {
            host: "localhost".to_string(),
            port: 5432,
            user: "testuser".to_string(),
            password: SecretString::new("password1".into()),
            url: None,
            ssl_mode: None,
            max_pool_size: None,
            connect_timeout_secs: None,
        };

        let config2 = PostgresConfig {
            password: SecretString::new("password2".into()),
            ..config1.clone()
        };

        assert_ne!(config1, config2);
    }

    #[test]
    fn test_connection_url_sqlite_default() {
        let config = DatabaseConfig::default();
        let url = config.connection_url().unwrap();
        assert_eq!(url, "sqlite:personal_ledger.db");
    }

    #[test]
    fn test_connection_url_sqlite_custom_database() {
        let config = DatabaseConfig {
            kind: domain::DbEngine::Sqlite,
            database: "custom_db".to_string(),
            postgres: None,
        };
        let url = config.connection_url().unwrap();
        assert_eq!(url, "sqlite:custom_db.db");
    }

    #[test]
    fn test_connection_url_sqlite_memory() {
        let config = DatabaseConfig {
            kind: domain::DbEngine::Sqlite,
            database: ":memory:".to_string(),
            postgres: None,
        };
        let url = config.connection_url().unwrap();
        assert_eq!(url, "sqlite::memory:.db");
    }

    #[test]
    fn test_connection_url_postgres_missing_config() {
        let config = DatabaseConfig {
            kind: domain::DbEngine::Postgres,
            database: "test_db".to_string(),
            postgres: None,
        };
        let result = config.connection_url();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ConfigError::Validation(_)));
        assert!(err.to_string().contains("postgres configuration missing"));
    }

    #[test]
    fn test_connection_url_postgres_with_url() {
        let config = DatabaseConfig {
            kind: domain::DbEngine::Postgres,
            database: "ignored_db".to_string(),
            postgres: Some(PostgresConfig {
                host: "ignored".to_string(),
                port: 0,
                user: "ignored".to_string(),
                password: SecretString::new("ignored".into()),
                url: Some("postgres://user:pass@host:5432/custom_db?sslmode=require".to_string()),
                ssl_mode: Some("ignored".to_string()),
                max_pool_size: None,
                connect_timeout_secs: None,
            }),
        };
        let url = config.connection_url().unwrap();
        assert_eq!(url, "postgres://user:pass@host:5432/custom_db?sslmode=require");
    }

    #[test]
    fn test_connection_url_postgres_composed_url() {
        let config = DatabaseConfig {
            kind: domain::DbEngine::Postgres,
            database: "test_db".to_string(),
            postgres: Some(PostgresConfig {
                host: "localhost".to_string(),
                port: 5432,
                user: "testuser".to_string(),
                password: SecretString::new("testpass".into()),
                url: None,
                ssl_mode: Some("require".to_string()),
                max_pool_size: None,
                connect_timeout_secs: None,
            }),
        };
        let url = config.connection_url().unwrap();
        assert_eq!(url, "postgres://testuser:testpass@localhost:5432/test_db?sslmode=require");
    }

    #[test]
    fn test_connection_url_postgres_no_ssl() {
        let config = DatabaseConfig {
            kind: domain::DbEngine::Postgres,
            database: "test_db".to_string(),
            postgres: Some(PostgresConfig {
                host: "localhost".to_string(),
                port: 5432,
                user: "testuser".to_string(),
                password: SecretString::new("testpass".into()),
                url: None,
                ssl_mode: None,
                max_pool_size: None,
                connect_timeout_secs: None,
            }),
        };
        let url = config.connection_url().unwrap();
        assert_eq!(url, "postgres://testuser:testpass@localhost:5432/test_db");
    }

    #[test]
    fn test_connection_url_postgres_different_ports() {
        let config = DatabaseConfig {
            kind: domain::DbEngine::Postgres,
            database: "test_db".to_string(),
            postgres: Some(PostgresConfig {
                host: "remote.host".to_string(),
                port: 9999,
                user: "admin".to_string(),
                password: SecretString::new("secret".into()),
                url: None,
                ssl_mode: Some("verify-full".to_string()),
                max_pool_size: None,
                connect_timeout_secs: None,
            }),
        };
        let url = config.connection_url().unwrap();
        assert_eq!(url, "postgres://admin:secret@remote.host:9999/test_db?sslmode=verify-full");
    }

    #[test]
    fn test_connection_url_postgres_special_characters() {
        let config = DatabaseConfig {
            kind: domain::DbEngine::Postgres,
            database: "test_db".to_string(),
            postgres: Some(PostgresConfig {
                host: "localhost".to_string(),
                port: 5432,
                user: "user@domain.com".to_string(),
                password: SecretString::new("pass%word!".into()),
                url: None,
                ssl_mode: None,
                max_pool_size: None,
                connect_timeout_secs: None,
            }),
        };
        let url = config.connection_url().unwrap();
        assert_eq!(url, "postgres://user@domain.com:pass%word!@localhost:5432/test_db");
    }

    #[test]
    fn test_database_config_clone() {
        let config = DatabaseConfig {
            kind: domain::DbEngine::Postgres,
            database: "test".to_string(),
            postgres: Some(PostgresConfig {
                host: "host".to_string(),
                port: 5432,
                user: "user".to_string(),
                password: SecretString::new("pass".into()),
                url: None,
                ssl_mode: None,
                max_pool_size: Some(5),
                connect_timeout_secs: Some(10),
            }),
        };

        let cloned = config.clone();
        assert_eq!(config, cloned);

        // Verify they are independent
        let url1 = config.connection_url().unwrap();
        let url2 = cloned.connection_url().unwrap();
        assert_eq!(url1, url2);
    }

    #[test]
    fn test_postgres_config_debug() {
        let config = PostgresConfig {
            host: "localhost".to_string(),
            port: 5432,
            user: "test".to_string(),
            password: SecretString::new("secret".into()),
            url: None,
            ssl_mode: Some("require".to_string()),
            max_pool_size: Some(10),
            connect_timeout_secs: Some(30),
        };

        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("localhost"));
        assert!(debug_str.contains("5432"));
        assert!(debug_str.contains("test"));
        // Note: password should be hidden in debug output due to secrecy
        assert!(debug_str.contains("require"));
        assert!(debug_str.contains("10"));
        assert!(debug_str.contains("30"));
    }

    #[test]
    fn test_database_config_debug() {
        let config = DatabaseConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("Sqlite"));
        assert!(debug_str.contains("personal_ledger"));
        assert!(debug_str.contains("None"));
    }

    #[test]
    fn test_connection_url_error_handling() {
        // Test that we handle the postgres config validation properly
        let config = DatabaseConfig {
            kind: domain::DbEngine::Postgres,
            database: "test".to_string(),
            postgres: None,
        };

        let result = config.connection_url();
        assert!(result.is_err());

        // Verify error message content
        if let Err(ConfigError::Validation(msg)) = result {
            assert!(msg.contains("postgres"));
            assert!(msg.contains("missing"));
        } else {
            panic!("Expected Validation error");
        }
    }

    #[test]
    fn test_database_config_with_postgres_config() {
        let postgres_config = PostgresConfig {
            host: "db.example.com".to_string(),
            port: 5432,
            user: "app_user".to_string(),
            password: SecretString::new("secure_password".into()),
            url: None,
            ssl_mode: Some("require".to_string()),
            max_pool_size: Some(20),
            connect_timeout_secs: Some(60),
        };

        let db_config = DatabaseConfig {
            kind: domain::DbEngine::Postgres,
            database: "ledger_prod".to_string(),
            postgres: Some(postgres_config),
        };

        assert_eq!(db_config.kind, domain::DbEngine::Postgres);
        assert_eq!(db_config.database, "ledger_prod");

        let url = db_config.connection_url().unwrap();
        assert_eq!(url, "postgres://app_user:secure_password@db.example.com:5432/ledger_prod?sslmode=require");
    }

    #[test]
    fn test_empty_database_name() {
        let config = DatabaseConfig {
            kind: domain::DbEngine::Sqlite,
            database: "".to_string(),
            postgres: None,
        };
        let url = config.connection_url().unwrap();
        assert_eq!(url, "sqlite:.db");
    }

    #[test]
    fn test_database_name_with_spaces() {
        let config = DatabaseConfig {
            kind: domain::DbEngine::Sqlite,
            database: "my database".to_string(),
            postgres: None,
        };
        let url = config.connection_url().unwrap();
        assert_eq!(url, "sqlite:my database.db");
    }

    #[test]
    fn test_postgres_config_minimal() {
        let config = PostgresConfig {
            host: "localhost".to_string(),
            port: 5432,
            user: "user".to_string(),
            password: SecretString::new("pass".into()),
            url: None,
            ssl_mode: None,
            max_pool_size: None,
            connect_timeout_secs: None,
        };

        let db_config = DatabaseConfig {
            kind: domain::DbEngine::Postgres,
            database: "test".to_string(),
            postgres: Some(config),
        };

        let url = db_config.connection_url().unwrap();
        assert_eq!(url, "postgres://user:pass@localhost:5432/test");
    }

    #[test]
    fn test_postgres_config_maximal() {
        let config = PostgresConfig {
            host: "secure-db.company.com".to_string(),
            port: 9999,
            user: "service_account".to_string(),
            password: SecretString::new("very_secure_password_123!".into()),
            url: None,
            ssl_mode: Some("verify-ca".to_string()),
            max_pool_size: Some(100),
            connect_timeout_secs: Some(120),
        };

        let db_config = DatabaseConfig {
            kind: domain::DbEngine::Postgres,
            database: "production_ledger".to_string(),
            postgres: Some(config),
        };

        let url = db_config.connection_url().unwrap();
        assert_eq!(url, "postgres://service_account:very_secure_password_123!@secure-db.company.com:9999/production_ledger?sslmode=verify-ca");
    }

    #[test]
    fn test_test_connection_url_sqlite() {
        // Test that test_connection_url can return SQLite URLs
        // Since it's random, we need to test multiple times to increase chances
        let config = DatabaseConfig::default();
        let mut found_sqlite = false;

        // Try up to 100 times to get a SQLite result
        for _ in 0..100 {
            if matches!(config.test_connection_url(), Ok(url) if url == "sqlite::memory:") {
                found_sqlite = true;
                break;
            }
        }

        assert!(found_sqlite, "test_connection_url should be able to return SQLite URLs");
    }

    #[test]
    fn test_test_connection_url_postgres_with_config() {
        // Test that test_connection_url can return Postgres URLs when config is available
        let postgres_config = PostgresConfig {
            host: "localhost".to_string(),
            port: 5432,
            user: "testuser".to_string(),
            password: SecretString::new("testpass".into()),
            url: None,
            ssl_mode: Some("require".to_string()),
            max_pool_size: None,
            connect_timeout_secs: None,
        };

        let config = DatabaseConfig {
            kind: domain::DbEngine::Postgres,
            database: "test_db".to_string(),
            postgres: Some(postgres_config),
        };

        let mut found_postgres = false;

        // Try up to 100 times to get a Postgres result
        for _ in 0..100 {
            if let Ok(url) = config.test_connection_url() &&
                url.starts_with("postgres://") && url.contains("testuser:testpass@localhost:5432/test_") {
                found_postgres = true;
                // Verify it uses a random database name starting with "test_"
                assert!(url.contains("test_"), "Postgres test URL should use random database name starting with 'test_'");
                break;
            }
        }

        assert!(found_postgres, "test_connection_url should be able to return Postgres URLs when config is available");
    }

    #[test]
    fn test_test_connection_url_postgres_with_url() {
        // Test that test_connection_url modifies full Postgres URLs with random database names
        let postgres_config = PostgresConfig {
            host: "localhost".to_string(),
            port: 5432,
            user: "testuser".to_string(),
            password: SecretString::new("testpass".into()),
            url: Some("postgres://user:pass@host:5432/original_db?sslmode=require".to_string()),
            ssl_mode: None,
            max_pool_size: None,
            connect_timeout_secs: None,
        };

        let config = DatabaseConfig {
            kind: domain::DbEngine::Postgres,
            database: "ignored".to_string(),
            postgres: Some(postgres_config),
        };

        let mut found_modified_url = false;

        // Try up to 100 times to get a Postgres result
        for i in 0..100 {
            if let Ok(url) = config.test_connection_url() {
                println!("Attempt {}: URL = {}", i, url);
                if url.starts_with("postgres://user:pass@host:5432/test_") && url.ends_with("?sslmode=require") {
                    found_modified_url = true;
                    // Verify the database name was replaced with a random one
                    assert!(url.contains("/test_"), "URL should have random database name");
                    assert!(!url.contains("/original_db"), "Original database name should be replaced");
                    break;
                }
            }
        }

        assert!(found_modified_url, "test_connection_url should modify full Postgres URLs with random database names");
    }

    #[test]
    fn test_test_connection_url_postgres_missing_config() {
        // Test that test_connection_url returns error when postgres is randomly selected but no config exists
        let config = DatabaseConfig {
            kind: domain::DbEngine::Sqlite, // Config says SQLite
            database: "test".to_string(),
            postgres: None, // But no postgres config
        };

        let mut found_error = false;

        // Try up to 100 times - some will succeed (SQLite), some will fail (Postgres without config)
        for _ in 0..100 {
            let result = config.test_connection_url();
            if matches!(result, Err(ConfigError::Validation(msg)) if msg.contains("postgres configuration missing for random postgres test")) {
                found_error = true;
                break;
            }
        }

        assert!(found_error, "test_connection_url should return error when postgres is randomly selected but no config exists");
    }

    #[test]
    fn test_test_connection_url_randomness() {
        // Test that test_connection_url actually produces different results over multiple calls
        let config = DatabaseConfig {
            kind: domain::DbEngine::Postgres,
            database: "test".to_string(),
            postgres: Some(PostgresConfig {
                host: "localhost".to_string(),
                port: 5432,
                user: "user".to_string(),
                password: SecretString::new("pass".into()),
                url: None,
                ssl_mode: None,
                max_pool_size: None,
                connect_timeout_secs: None,
            }),
        };

        let mut urls = std::collections::HashSet::new();

        // Collect results from multiple calls
        for _ in 0..50 {
            if let Ok(url) = config.test_connection_url() {
                urls.insert(url);
            }
        }

        // We should get at least 2 different URLs (SQLite memory + various Postgres random DBs)
        assert!(urls.len() >= 2, "test_connection_url should produce different results over multiple calls, got {} unique URLs", urls.len());
    }
}
