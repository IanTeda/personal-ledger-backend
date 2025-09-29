//! Database configuration and connection helpers.
//!
//! This module defines the `DatabaseConfig` domain type and helpers for
//! creating `sqlx` connection pools for the configured engine. The current
//! implementation supports Postgres and Sqlite.
//!
//! Defaults are intentionally conservative (in-memory sqlite) to make tests
//! and local development simple; production deployments should provide a
//! Postgres configuration via the `postgres` section or the `url` field.

// Re-export the secrecy trait used when composing connection URLs.
use secrecy::ExposeSecret;

use crate::config::ConfigError;


/// Default engine used when no database configuration is provided.
///
/// This defaults to `DbEngine::Sqlite` which keeps local development and
/// tests simple; production deployments should provide a Postgres config.
pub const DEFAULT_DB_ENGINE: DbEngine = DbEngine::Sqlite;

/// Default database name used when no value is provided by file or env.
/// This defaults to "personal_ledger" to match the application name.
/// Production deployments should override this to
pub const DEFAULT_DATABASE: &str = "personal_ledger";

/// Default Postgres configuration (none by default).
pub const DEFAULT_POSTGRES: Option<PostgresConfig> = None;

/// Database engine selector.
///
/// This enum is used by `DatabaseConfig` to determine which database backend
/// should be used at runtime. Values are deserialised case-insensitively
/// (lowercase) from configuration sources.
/// 
/// TODO: Move to domain module
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DbEngine {
    /// Use a Postgres server via `sqlx::PgPool`.
    Postgres,
    /// Use an embedded SQLite database via `sqlx::SqlitePool`.
    Sqlite,
}

impl std::fmt::Display for DbEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DbEngine::Postgres => "postgres",
            DbEngine::Sqlite => "sqlite",
        };
        write!(f, "{}", s)
    }
}

/// Configuration required to connect to a Postgres server.
///
/// The optional `url` field can be used to provide a full connection URL
/// (e.g. `postgres://user:pass@host:5432/dbname`). If `url` is present it
/// will be preferred over composing a URL from the other fields.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PostgresConfig {
    /// Postgres host name or IP address.
    pub host: String,

    /// TCP port for the Postgres server (usually 5432).
    pub port: u16,

    /// Username used to authenticate.
    pub user: String,

    /// Password stored as a secret string (not logged).
    pub password: secrecy::SecretString,

    /// Optional full connection URL (preferred if present).
    ///
    /// Example: `postgres://user:pass@host:5432/dbname`.
    pub url: Option<String>,

    /// Optional SSL mode passed as `sslmode` query parameter when composing
    /// a URL (for example `disable`, `prefer`, `require`).
    pub ssl_mode: Option<String>,

    /// Optional maximum number of connections in the pool.
    pub max_pool_size: Option<u32>,

    /// Optional connection timeout (seconds) used when creating the pool.
    pub connect_timeout_secs: Option<u64>,
}

/// Top-level database configuration.
///
/// `DatabaseConfig` selects the engine (via `kind`) and contains optional
/// per-engine settings (for example `postgres` for `DbEngine::Postgres`).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct DatabaseConfig {
    /// The database engine to use (postgres or sqlite).
    pub kind: DbEngine,

    /// Name of the database to connect to.
    pub database: String,

    /// Postgres-specific configuration (used when `kind == DbEngine::Postgres`).
    pub postgres: Option<PostgresConfig>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            kind: DEFAULT_DB_ENGINE,
            database: DEFAULT_DATABASE.to_string(),
            postgres: DEFAULT_POSTGRES,
        }
    }
}

/// Wrapper around the concrete `sqlx` connection pool types returned by
/// `DatabaseConfig::connection_pool`.
///
/// Matching on this enum gives access to the concrete pool (either
/// `sqlx::SqlitePool` or `sqlx::PgPool`) so callers can use the appropriate
/// API for their queries.
/// 
/// TODO: Move to domain module
#[derive(Debug, Clone)]
pub enum ConnectionPool {
    /// A `sqlx::SqlitePool` instance.
    Sqlite(sqlx::SqlitePool),

    /// A `sqlx::PgPool` instance.
    Postgres(sqlx::PgPool),
}

impl DatabaseConfig {
    /// Create a connection pool for the configured database engine.
    ///
    /// Returns a `ConnectionPool` enum wrapping either a `sqlx::SqlitePool`
    /// or `sqlx::PgPool`. This function is `async` because establishing the
    /// underlying pool requires asynchronous I/O.
    ///
    /// Errors are returned as a boxed `dyn std::error::Error` for simplicity;
    /// callers can inspect the error string or change this to a concrete
    /// error type if desired. Production-ready code should wire `max_pool_size`
    /// and `connect_timeout_secs` into `sqlx::PoolOptions` instead of using
    /// the defaults used here.
    /// 
    /// TODO: Improve error handling by creating the missing database file with
    /// `sqlx::SqlitePool::connect`.
    pub async fn connection_pool(
        &self,
    ) -> Result<ConnectionPool, ConfigError> {
        match self.kind {
            DbEngine::Sqlite => {
                // Use the configured database name as the SQLite file path.
                // For in-memory, set database to ":memory:" in config.
                let url = if self.database == ":memory:" {
                    "sqlite::memory:".to_string()
                } else {
                    format!("sqlite:{}.db", self.database)
                };
                let pool = sqlx::SqlitePool::connect(&url).await
                    .map_err(|e| ConfigError::Validation(format!("SQLite connection failed: {}", e)))?;
                Ok(ConnectionPool::Sqlite(pool))
            }
            DbEngine::Postgres => {
                // Ensure we have a Postgres config
                let pg = match &self.postgres {
                    Some(cfg) => cfg,
                    None => {
                        return Err(ConfigError::Validation(
                            "postgres configuration missing".to_string(),
                        ));
                    }
                };

                let url = if let Some(u) = &pg.url {
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
                };

                let pool = sqlx::PgPool::connect(&url).await
                    .map_err(|e| ConfigError::Validation(format!("Postgres connection failed: {}", e)))?;
                Ok(ConnectionPool::Postgres(pool))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_sqlite() {
        let cfg = DatabaseConfig::default();
        assert!(matches!(cfg.kind, DbEngine::Sqlite));
        assert_eq!(cfg.database, DEFAULT_DATABASE);
        assert!(cfg.postgres.is_none());
    }

    #[tokio::test]
    async fn sqlite_connection_pool_works() {
        let cfg = DatabaseConfig {
            kind: DbEngine::Sqlite,
            database: ":memory:".to_string(),
            postgres: None,
        };

        let pool = cfg.connection_pool().await.unwrap();
        match pool {
            ConnectionPool::Sqlite(p) => {
                // Simple query to verify the pool works
                let row: (i64,) = sqlx::query_as("SELECT 1")
                    .fetch_one(&p)
                    .await
                    .unwrap();
                assert_eq!(row.0, 1);
            }
            _ => panic!("expected sqlite pool"),
        }
    }

    #[tokio::test]
    async fn postgres_missing_config_returns_error() {
        let cfg = DatabaseConfig {
            kind: DbEngine::Postgres,
            database: DEFAULT_DATABASE.to_string(),
            postgres: None,
        };

        let res = cfg.connection_pool().await;
        assert!(res.is_err());
        if let Err(e) = res {
            let s = format!("{}", e);
            assert!(s.contains("postgres configuration missing"));
        }
    }

    #[test]
    fn dbengine_display() {
        assert_eq!(DbEngine::Postgres.to_string(), "postgres");
        assert_eq!(DbEngine::Sqlite.to_string(), "sqlite");
    }

    #[test]
    fn default_postgres_is_none() {
        let def = DatabaseConfig::default();
        assert!(def.postgres.is_none(), "default Postgres config should be None");
    }

    #[tokio::test]
    async fn postgres_invalid_url_fails() {
        let pg = PostgresConfig {
            host: "127.0.0.1".into(),
            port: 1, // Invalid port
            user: "user".into(),
            password: secrecy::SecretString::new("pass".into()),
            url: Some("postgres://user:pass@127.0.0.1:1/db".into()),
            ssl_mode: None,
            max_pool_size: None,
            connect_timeout_secs: Some(1),
        };

        let cfg = DatabaseConfig {
            kind: DbEngine::Postgres,
            database: "db".into(),
            postgres: Some(pg),
        };

        let res = cfg.connection_pool().await;
        assert!(res.is_err(), "expected connection_pool to return Err for invalid/unreachable URL");
    }
}