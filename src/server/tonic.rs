use core::net;
use tokio::net as TokioNet;
use tokio_stream::wrappers::TcpListenerStream;

use crate::{server, LedgerResult};

/// TonicServer provides a higher-level abstraction for running a Tonic gRPC server.
/// It combines a Router with a TcpListener for more control over server lifecycle,
/// particularly useful for integration testing with random port binding.
pub struct TonicServer {
    pub router: server::Router,
    pub listener: TokioNet::TcpListener,
}

impl TonicServer {
    /// Create a new TonicServer instance with the given address.
    /// If port is 0, the OS will assign a random available port.
    pub async fn new(address: net::SocketAddr) -> LedgerResult<Self> {
        let router = server::Router::new().await?;
        let listener = TokioNet::TcpListener::bind(address).await?;

        Ok(Self { router, listener })
    }

    /// Get the local address that the server is bound to.
    pub fn local_addr(&self) -> Result<net::SocketAddr, std::io::Error> {
        self.listener.local_addr()
    }

    /// Get a formatted string representation of the server address.
    pub fn address_string(&self) -> LedgerResult<String> {
        let addr = self.local_addr()?;
        Ok(format!("{}:{}", addr.ip(), addr.port()))
    }

    /// Run the server using the bound listener.
    /// This consumes the TonicServer instance.
    pub async fn run(self) -> LedgerResult<()> {
        let addr_string = self.address_string()?;
        tracing::info!("Tonic server listening on {}", addr_string);

        let incoming = TcpListenerStream::new(self.listener);
        let router = self.router;
        router.into_inner().serve_with_incoming(incoming).await?;

        Ok(())
    }

    /// Run the server on a specific address.
    /// This is a convenience method that creates a new listener and runs the server.
    pub async fn run_on_addr(self, addr: net::SocketAddr) -> Result<(), tonic::transport::Error> {
        self.serve(addr).await
    }

    /// Start serving the router on the given address.
    ///
    /// # Arguments
    /// * `addr` - The socket address to bind the server to.
    ///
    /// # Errors
    /// Returns a tonic transport error if the server fails to start.
    pub async fn serve(self, addr: std::net::SocketAddr) -> Result<(), tonic::transport::Error> {
        self.router.into_inner().serve(addr).await
    }

    /// Start serving with an existing incoming stream.
    ///
    /// This is useful for integration tests or advanced scenarios where you want to control the incoming connections.
    ///
    /// # Arguments
    /// * `incoming` - A stream of incoming TCP connections.
    ///
    /// # Errors
    /// Returns a tonic transport error if the server fails to start.
    pub async fn serve_with_incoming<I>(self, incoming: I) -> Result<(), tonic::transport::Error>
    where
        I: futures_util::stream::Stream<Item = Result<tokio::net::TcpStream, std::io::Error>> + Send + 'static,
    {
        self.router.into_inner().serve_with_incoming(incoming).await
    }

    /// Get a reference to the health reporter for health status management.
    pub fn health_reporter(&self) -> &tonic_health::server::HealthReporter {
        self.router.health_reporter()
    }

    /// Set the health status for a specific service.
    pub async fn set_service_health<S>(&self, serving: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        S: tonic::server::NamedService,
    {
        self.router.set_service_health::<S>(serving).await
    }
}

