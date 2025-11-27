use core::net;
use std::sync::Arc;
use tokio::sync::oneshot;

use personal_ledger_backend::{server, telemetry, LedgerConfig};

/// Error type for server spawning operations
pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, Error>;

// Ensure that the `tracing` stack is only initialised once using `once_cell`
// Lazy makes it globally available
static TRACING: once_cell::sync::Lazy<()> = once_cell::sync::Lazy::new(|| {
    // Use ERROR level for integration tests to reduce noise, but allow override via env
    let telemetry_level = std::env::var("RUST_LOG")
        .map(|level| level.parse().unwrap_or(telemetry::LogLevel::ERROR))
        .unwrap_or(telemetry::LogLevel::ERROR);
    let _telemetry = telemetry::init(telemetry_level);
});

/// A test gRPC server that can be spawned for integration testing.
///
/// This struct manages the lifecycle of a test gRPC server, providing
/// methods to start the server and obtain connection channels for clients.
#[derive(Clone)]
pub struct SpawnTonicServer {
    /// The actual address the server is bound to (with OS-assigned port)
    pub address: net::SocketAddr,
    /// Shutdown sender to gracefully stop the server
    #[allow(dead_code)]
    shutdown_tx: Arc<tokio::sync::Mutex<Option<oneshot::Sender<()>>>>,
}

impl SpawnTonicServer {
    /// Initialize a new test gRPC server with the given database pool.
    ///
    /// This method:
    /// 1. Initializes tracing for test logging
    /// 2. Parses configuration and sets port to 0 for auto-assignment
    /// 3. Creates and starts the gRPC server in the background
    /// 4. Waits for the server to be ready
    /// 5. Returns the server instance with the actual bound address
    ///
    /// # Arguments
    /// * `database_pool` - The SQLite database pool for the server
    ///
    /// # Returns
    /// A `SpawnTonicServer` instance ready for client connections
    ///
    /// # Errors
    /// Returns an error if configuration parsing, server creation, or startup fails
    pub async fn init(database_pool: sqlx::SqlitePool) -> Result<Self> {
        // Initialize tracing for integration testing
        once_cell::sync::Lazy::force(&TRACING);

        // Parse configuration and modify for testing
        let mut ledger_config = LedgerConfig::parse()?;
        // Use port 0 to let OS assign an available port, avoiding conflicts
        ledger_config.server.port = 0;

        // Create the server instance
        let server = server::TonicServer::new(database_pool, ledger_config).await?;
        let actual_address = server.local_addr()?;

        // Create shutdown channel for graceful server termination
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        // Spawn server in background task
        tokio::spawn(async move {
            let server_future = server.run();
            let shutdown_future = shutdown_rx;

            tokio::select! {
                result = server_future => {
                    if let Err(e) = result {
                        tracing::error!("Test server error: {}", e);
                    }
                }
                _ = shutdown_future => {
                    tracing::info!("Test server shutting down gracefully");
                }
            }
        });

        // Wait for server to be ready (with timeout)
        Self::wait_for_server_ready(&actual_address).await?;

        Ok(Self {
            address: actual_address,
            shutdown_tx: Arc::new(tokio::sync::Mutex::new(Some(shutdown_tx))),
        })
    }

    /// Wait for the server to be ready by attempting connections.
    ///
    /// This method tries to establish a connection to verify the server is listening.
    /// It uses a timeout to avoid hanging if the server fails to start.
    async fn wait_for_server_ready(address: &net::SocketAddr) -> Result<()> {
        let max_attempts = 50; // 5 seconds max wait
        let attempt_delay = std::time::Duration::from_millis(100);

        for attempt in 1..=max_attempts {
            match tokio::net::TcpStream::connect(address).await {
                Ok(_) => {
                    tracing::debug!("Test server ready on {}", address);
                    return Ok(());
                }
                Err(_) => {
                    if attempt == max_attempts {
                        return Err(format!("Server failed to start on {} after {} attempts", address, max_attempts).into());
                    }
                    tokio::time::sleep(attempt_delay).await;
                }
            }
        }

        unreachable!("Loop should return or error")
    }

    /// Get the address the server is bound to.
    ///
    /// This returns the actual address with the OS-assigned port.
    #[allow(dead_code)]
    pub fn address(&self) -> net::SocketAddr {
        self.address
    }

    /// Create a tonic transport channel for connecting to this server.
    ///
    /// This method creates a lazy-connecting channel that can be used
    /// by tonic clients to communicate with the test server.
    ///
    /// # Returns
    /// A `tonic::transport::Channel` configured for this server
    pub fn transport_channel(&self) -> tonic::transport::Channel {
        let server_uri = format!("http://{}", self.address);
        tonic::transport::Channel::from_shared(server_uri)
            .expect("Server address should be valid URI")
            .connect_lazy()
    }

    /// Gracefully shut down the test server.
    ///
    /// This method signals the server to stop and waits for it to terminate.
    /// Should be called in test cleanup to ensure proper resource cleanup.
    ///
    /// # Note
    /// This is a best-effort shutdown. If the server doesn't respond to the
    /// shutdown signal within a reasonable time, the test process may still exit.
    #[allow(dead_code)]
    pub async fn shutdown(self) {
        if let Some(tx) = self.shutdown_tx.lock().await.take() {
            let _ = tx.send(());
            // Give server time to shut down gracefully
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
}