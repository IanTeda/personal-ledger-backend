

/// Database engine selector.
///
/// This enum is used by `DatabaseConfig` to determine which database backend
/// should be used at runtime. Values are deserialised case-insensitively
/// (lowercase) from configuration sources.
/// 
/// TODO: Move to domain module
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
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