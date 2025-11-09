//-- ./src/main.rs

use personal_ledger_backend::{database, server, telemetry, LedgerResult};

/// Main server implementation for personal-ledger-backend
/// Implements a gRPC server using Tonic and includes server reflection
/// The server currently implements a simple UtilitiesService with a Ping method to demonstrate functionality.
#[tokio::main]
async fn main() -> LedgerResult<()> {

    // Parse ledger configuration file
    let ledger_config = personal_ledger_backend::LedgerConfig::parse()?;

    let log_level = ledger_config.server.log_level();

    #[allow(clippy::let_unit_value)]
    let _telemetry_guard = telemetry::init(log_level)?;
    tracing::info!("Starting tracing at level '{:?}'", log_level);

    // Initialize the database connection pool and run migrations
    let database_url = ledger_config.server.database_url()?;
    let database_pool = database::DatabasePool::new(&database_url);
    let database = database_pool.connect().await?;

    let tonic_server = server::TonicServer::new(database.into_pool()?, ledger_config).await?;

    tonic_server.run().await?;

    Ok(())
}