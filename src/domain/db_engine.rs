/// Supported database engine types for the Personal Ledger backend.
///
/// This enum is used by [`DatabaseConfig`](crate::config::DatabaseConfig) to determine
/// which database backend should be used at runtime. Values are deserialised case-insensitively
/// (lowercase) from configuration sources (e.g., environment variables, config files).
///
/// - `Postgres`: Use a Postgres server via [`sqlx::PgPool`].
/// - `Sqlite`: Use an embedded SQLite database via [`sqlx::SqlitePool`].
///
/// # Example
///
/// ```
/// use personal_ledger_backend::domain::DbEngine;
/// let engine: DbEngine = serde_json::from_str(r#"postgres"#).unwrap();
/// assert_eq!(engine, DbEngine::Postgres);
/// ```
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DbEngine {
    /// Use a Postgres server via `sqlx::PgPool`.
    Postgres,
    /// Use an embedded SQLite database via `sqlx::SqlitePool`.
    Sqlite,
}

impl std::fmt::Display for DbEngine {
    /// Returns the canonical string name for the database engine.
    ///
    /// # Example
    ///
    /// ```
    /// use personal_ledger_backend::domain::DbEngine;
    /// assert_eq!(DbEngine::Postgres.to_string(), "postgres");
    /// assert_eq!(DbEngine::Sqlite.to_string(), "sqlite");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DbEngine::Postgres => "postgres",
            DbEngine::Sqlite => "sqlite",
        };
        write!(f, "{}", s)
    }
}

impl DbEngine {
    /// Returns a random database engine variant (`DbEngine::Postgres` or `DbEngine::Sqlite`).
    ///
    /// # Example
    ///
    /// ```
    /// use personal_ledger_backend::domain::DbEngine;
    /// let engine = DbEngine::random();
    /// assert!(matches!(engine, DbEngine::Postgres | DbEngine::Sqlite));
    /// ```
    ///
    /// The selection is uniform and non-deterministic.
    pub fn random() -> Self {
        use rand::Rng;
        let mut rng = rand::rng();
        match rng.random_range(0..2) {
            0 => DbEngine::Postgres,
            _ => DbEngine::Sqlite,
        }
    }
}

