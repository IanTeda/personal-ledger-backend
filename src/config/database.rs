// const DEFAULT_DB_ENGINE: &str = "sqlite";

// const DEFAULT_POSTGRES: Option<PostgresConfig> = None;


// #[derive(Debug, Clone, serde::Deserialize)]
// pub enum DbEngine {
//     Postgres,
//     Sqlite,
// }

// #[derive(Debug, Clone, serde::Deserialize)]
// pub struct PostgresConfig {
//     pub host: String,
//     pub port: u16,
//     pub user: String,
//     pub password: secrecy::SecretString,
//     pub database: String,
//     /// Optional full connection URL; if provided, use this instead of composing from host/user.
//     pub url: Option<String>,
//     pub ssl_mode: Option<String>, // "disable"|"prefer"|"require" etc
//     pub max_pool_size: Option<u32>,
//     pub connect_timeout_secs: Option<u64>,
// }

// #[derive(Debug, Clone, serde::Deserialize)]
// pub struct DatabaseConfig {
//     pub kind: DbEngine,
//     pub postgres: Option<PostgresConfig>,
// }


// impl Default for DatabaseConfig {
//     fn default() -> Self {
//         Self {
//             kind: DbEngine::Sqlite,
//             postgres: None,
//         }
//     }
// }