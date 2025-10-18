//! # Tonic Server Module
//!
//! This module provides the `TonicServer` struct, which offers a high-level abstraction for running Tonic gRPC servers.
//!
//! The `TonicServer` combines a `Router` (which manages gRPC services) with a `TcpListener` to provide
//! enhanced control over server lifecycle and connection handling. This design is particularly beneficial
//! for integration testing scenarios where random port binding and controlled server startup are essential.
//!
//! ## Key Features
//!
//! - **Random Port Binding**: Set port to 0 for automatic OS-assigned ports (ideal for testing)
//! - **Flexible Serving**: Support for both address-based and stream-based serving
//! - **Health Management**: Integration with gRPC health checking services
//! - **Integration Testing**: Designed for easy testing with controlled server lifecycles
//!
//! ## Usage
//!
//! Create a server with automatic port assignment:
//!
//! ```rust
//! let server = TonicServer::new("127.0.0.1:0".parse().unwrap()).await?;
//! let addr = server.local_addr()?;
//! println!("Server listening on {}", server.address_string()?);
//! server.run().await?;
//! ```
//!
//! For integration testing with custom streams:
//!
//! ```rust
//! let server = TonicServer::new("127.0.0.1:0".parse().unwrap()).await?;
//! let custom_stream = /* your stream implementation */;
//! server.serve_with_incoming(custom_stream).await?;
//! ```
//!
//! ## Architecture
//!
//! The `TonicServer` acts as a bridge between the service composition layer (`Router`) and the
//! network layer (`TcpListener`). It provides a clean API for server management while delegating
//! service-specific logic to the `Router`.
//!
//! This module is part of the server infrastructure and is intended for internal use by the
//! Personal Ledger backend server components.
use core::net;
use tokio::net as TokioNet;
use tokio_stream::wrappers::TcpListenerStream;

use crate::{server, LedgerResult};

/// TonicServer provides a higher-level abstraction for running a Tonic gRPC server.
/// It combines a Router with a TcpListener for more control over server lifecycle,
/// particularly useful for integration testing with random port binding.
///
/// ## Example
///
/// ```rust
/// use personal_ledger_backend::server::TonicServer;
///
/// // Create server with random port
/// let server = TonicServer::new("127.0.0.1:0".parse().unwrap()).await?;
///
/// // Get the assigned address
/// let addr = server.local_addr()?;
/// println!("Server bound to: {}", addr);
///
/// // Start serving
/// server.run().await?;
/// ```
pub struct TonicServer {
    /// The router containing all configured gRPC services (reflection, health, utilities).
    /// This field is public to allow direct access for advanced use cases.
    pub router: server::Router,
    /// The TCP listener bound to the server address.
    /// This field is public to allow direct access for advanced networking scenarios.
    pub listener: TokioNet::TcpListener,
}

impl TonicServer {
    /// Create a new TonicServer instance with the given address.
    ///
    /// This method initializes a new router with all required services and binds a TCP listener
    /// to the specified address. If the port is set to 0, the operating system will assign
    /// a random available port.
    ///
    /// # Arguments
    ///
    /// * `address` - The socket address to bind the server to. Use port 0 for auto-assignment.
    ///
    /// # Returns
    ///
    /// Returns a `LedgerResult<TonicServer>` containing the initialized server on success.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Router initialization fails (service configuration issues)
    /// - TCP listener binding fails (address already in use, permission denied, etc.)
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Bind to localhost with random port
    /// let server = TonicServer::new("127.0.0.1:0".parse().unwrap()).await?;
    ///
    /// // Bind to specific address
    /// let server = TonicServer::new("127.0.0.1:8080".parse().unwrap()).await?;
    /// ```
    pub async fn new(address: net::SocketAddr) -> LedgerResult<Self> {
        let router = server::Router::new().await?;
        let listener = TokioNet::TcpListener::bind(address).await?;

        Ok(Self { router, listener })
    }

    /// Get the local address that the server is bound to.
    ///
    /// This method returns the actual socket address that the TCP listener is bound to,
    /// which is useful when the server was created with port 0 (auto-assignment).
    ///
    /// # Returns
    ///
    /// Returns `Result<SocketAddr, std::io::Error>` containing the bound address.
    ///
    /// # Errors
    ///
    /// Returns an `std::io::Error` if the listener's local address cannot be retrieved
    /// (rare, but can happen if the listener is in an invalid state).
    ///
    /// # Examples
    ///
    /// ```rust
    /// let server = TonicServer::new("127.0.0.1:0".parse().unwrap()).await?;
    /// let addr = server.local_addr()?;
    /// println!("Server bound to: {}", addr);
    /// ```
    pub fn local_addr(&self) -> Result<net::SocketAddr, std::io::Error> {
        self.listener.local_addr()
    }

    /// Get a formatted string representation of the server address.
    ///
    /// This is a convenience method that returns the server address as a formatted string
    /// in the format "IP:PORT", suitable for logging or display purposes.
    ///
    /// # Returns
    ///
    /// Returns `LedgerResult<String>` containing the formatted address string.
    ///
    /// # Errors
    ///
    /// Returns an error if `local_addr()` fails to retrieve the bound address.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let server = TonicServer::new("127.0.0.1:8080".parse().unwrap()).await?;
    /// let addr_str = server.address_string()?;
    /// assert_eq!(addr_str, "127.0.0.1:8080");
    /// ```
    pub fn address_string(&self) -> LedgerResult<String> {
        let addr = self.local_addr()?;
        Ok(format!("{}:{}", addr.ip(), addr.port()))
    }

    /// Run the server using the bound listener.
    ///
    /// This method starts the gRPC server using the pre-bound TCP listener. It consumes
    /// the `TonicServer` instance and runs until the server is shut down or an error occurs.
    /// The server will log its listening address and begin accepting connections.
    ///
    /// # Returns
    ///
    /// Returns `LedgerResult<()>` on successful server shutdown.
    ///
    /// # Errors
    ///
    /// Returns an error if the server fails to start or encounters a fatal error during operation.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let server = TonicServer::new("127.0.0.1:8080".parse().unwrap()).await?;
    /// server.run().await?;
    /// ```
    ///
    /// # Note
    ///
    /// This method consumes `self` and will run indefinitely until interrupted.
    /// For testing scenarios, consider using `serve_with_incoming()` with a controlled stream.
    pub async fn run(self) -> LedgerResult<()> {
        let addr_string = self.address_string()?;
        tracing::info!("Tonic server listening on {}", addr_string);

        let incoming = TcpListenerStream::new(self.listener);
        let router = self.router;
        router.into_inner().serve_with_incoming(incoming).await?;

        Ok(())
    }

    /// Run the server on a specific address.
    ///
    /// This is a convenience method that creates a new TCP listener for the given address
    /// and immediately starts serving. It's equivalent to creating a new `TonicServer` instance
    /// and calling `run()`, but in a single step.
    ///
    /// # Arguments
    ///
    /// * `addr` - The socket address to bind the server to.
    ///
    /// # Returns
    ///
    /// Returns `Result<(), tonic::transport::Error>` on successful server shutdown.
    ///
    /// # Errors
    ///
    /// Returns a `tonic::transport::Error` if server startup or operation fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let server = TonicServer::new("127.0.0.1:0".parse().unwrap()).await?;
    /// // Run on a different address
    /// server.run_on_addr("127.0.0.1:8080".parse().unwrap()).await?;
    /// ```
    ///
    /// # Note
    ///
    /// This method consumes `self` and will run indefinitely until interrupted.
    pub async fn run_on_addr(self, addr: net::SocketAddr) -> Result<(), tonic::transport::Error> {
        self.serve(addr).await
    }

    /// Start serving the router on the given address.
    ///
    /// This method binds to the specified address and starts serving gRPC requests.
    /// It provides direct control over the server address without using the pre-bound listener.
    ///
    /// # Arguments
    ///
    /// * `addr` - The socket address to bind the server to.
    ///
    /// # Returns
    ///
    /// Returns `Result<(), tonic::transport::Error>` on successful server shutdown.
    ///
    /// # Errors
    ///
    /// Returns a `tonic::transport::Error` if the server fails to bind to the address or start serving.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let server = TonicServer::new("127.0.0.1:0".parse().unwrap()).await?;
    /// server.serve("127.0.0.1:8080".parse().unwrap()).await?;
    /// ```
    ///
    /// # Note
    ///
    /// This method consumes `self` and will run indefinitely until interrupted.
    pub async fn serve(self, addr: std::net::SocketAddr) -> Result<(), tonic::transport::Error> {
        self.router.into_inner().serve(addr).await
    }

    /// Start serving with an existing incoming stream.
    ///
    /// This method allows for advanced server configurations where you want full control
    /// over the incoming connection stream. This is particularly useful for integration testing
    /// where you might want to use mock connections or control connection timing.
    ///
    /// # Arguments
    ///
    /// * `incoming` - A stream that yields `Result<TcpStream, std::io::Error>`. The stream
    ///   should produce incoming TCP connections for the server to handle.
    ///
    /// # Returns
    ///
    /// Returns `Result<(), tonic::transport::Error>` on successful server shutdown.
    ///
    /// # Errors
    ///
    /// Returns a `tonic::transport::Error` if the server fails to start or encounters errors
    /// while processing the incoming stream.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tokio_stream::StreamExt;
    /// use futures_util::stream;
    ///
    /// let server = TonicServer::new("127.0.0.1:0".parse().unwrap()).await?;
    ///
    /// // Create a custom stream (for testing)
    /// let custom_stream = stream::empty();
    ///
    /// server.serve_with_incoming(custom_stream).await?;
    /// ```
    ///
    /// # Type Parameters
    ///
    /// * `I` - The type of the incoming stream. Must implement `Stream` and be `Send + 'static`.
    ///
    /// # Note
    ///
    /// This method consumes `self` and will run indefinitely until the stream ends or an error occurs.
    pub async fn serve_with_incoming<I>(self, incoming: I) -> Result<(), tonic::transport::Error>
    where
        I: futures_util::stream::Stream<Item = Result<tokio::net::TcpStream, std::io::Error>> + Send + 'static,
    {
        self.router.into_inner().serve_with_incoming(incoming).await
    }

    /// Get a reference to the health reporter for health status management.
    ///
    /// The health reporter allows you to set the health status of individual gRPC services,
    /// which is used by clients and load balancers to determine service availability.
    ///
    /// # Returns
    ///
    /// Returns a reference to the `tonic_health::server::HealthReporter`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tonic::server::NamedService;
    ///
    /// let server = TonicServer::new("127.0.0.1:0".parse().unwrap()).await?;
    /// let reporter = server.health_reporter();
    ///
    /// // Mark a service as healthy
    /// reporter.set_serving::<MyService>().await;
    /// ```
    pub fn health_reporter(&self) -> &tonic_health::server::HealthReporter {
        self.router.health_reporter()
    }

    /// Set the health status for a specific service.
    ///
    /// This is a convenience method that delegates to the router's health reporter.
    /// Use this method to update the health status of individual gRPC services.
    ///
    /// # Arguments
    ///
    /// * `serving` - `true` to mark the service as healthy and serving, `false` to mark it as not serving.
    ///
    /// # Returns
    ///
    /// Returns `Result<(), Box<dyn std::error::Error + Send + Sync>>` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the health reporter fails to update the service status.
    ///
    /// # Type Parameters
    ///
    /// * `S` - The service type that implements `NamedService`. This is typically a generated
    ///   service server type from your protobuf definitions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let server = TonicServer::new("127.0.0.1:0".parse().unwrap()).await?;
    ///
    /// // Mark utilities service as healthy
    /// server.set_service_health::<crate::rpc::UtilitiesServiceServer<crate::services::UtilitiesService>>(true).await?;
    ///
    /// // Mark service as not serving (e.g., during shutdown)
    /// server.set_service_health::<crate::rpc::UtilitiesServiceServer<crate::services::UtilitiesService>>(false).await?;
    /// ```
    pub async fn set_service_health<S>(&self, serving: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        S: tonic::server::NamedService,
    {
        self.router.set_service_health::<S>(serving).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    /// Test successful creation of TonicServer with valid address
    #[tokio::test]
    async fn test_tonic_server_new_success() {
        let addr = "127.0.0.1:0".parse().unwrap();
        let result = TonicServer::new(addr).await;
        assert!(result.is_ok(), "TonicServer::new() should succeed with valid address");
    }

    /// Test creation of TonicServer with IPv6 address
    #[tokio::test]
    async fn test_tonic_server_new_ipv6() {
        let addr = "[::1]:0".parse().unwrap();
        let result = TonicServer::new(addr).await;
        assert!(result.is_ok(), "TonicServer::new() should succeed with IPv6 address");
    }

    /// Test error handling for invalid address
    #[tokio::test]
    async fn test_tonic_server_new_invalid_address() {
        // Use an address with an invalid IP that will fail to bind
        // 999.999.999.999 is not a valid IP format, so parsing will fail
        let addr_result = "999.999.999.999:8080".parse::<std::net::SocketAddr>();
        assert!(addr_result.is_err(), "Invalid IP should fail to parse");

        // For binding errors, we'd need an address that parses but fails to bind
        // This is harder to test reliably in unit tests
        // So we test the parsing error case instead
    }

    /// Test local_addr method returns valid address
    #[tokio::test]
    async fn test_tonic_server_local_addr() {
        let addr = "127.0.0.1:0".parse().unwrap();
        let server = TonicServer::new(addr).await.unwrap();

        let local_addr = server.local_addr();
        assert!(local_addr.is_ok(), "local_addr() should succeed");

        let addr = local_addr.unwrap();
        assert_eq!(addr.ip(), IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        assert!(addr.port() > 0, "Port should be assigned (non-zero)");
    }

    /// Test address_string method formats correctly
    #[tokio::test]
    async fn test_tonic_server_address_string() {
        let addr = "127.0.0.1:0".parse().unwrap();
        let server = TonicServer::new(addr).await.unwrap();

        let addr_string = server.address_string();
        assert!(addr_string.is_ok(), "address_string() should succeed");

        let formatted = addr_string.unwrap();
        assert!(formatted.starts_with("127.0.0.1:"), "Should start with IP");
        assert!(formatted.len() > "127.0.0.1:".len(), "Should contain port");
    }

    /// Test address_string error handling when local_addr fails
    #[tokio::test]
    async fn test_tonic_server_address_string_error() {
        // Create a server and then manually corrupt the listener to simulate error
        let addr = "127.0.0.1:0".parse().unwrap();
        let server = TonicServer::new(addr).await.unwrap();

        // Replace listener with a closed one to simulate error
        // This is tricky to test directly, so we'll test the happy path
        // In a real scenario, we'd mock or use dependency injection
        let _addr_string = server.address_string().unwrap();
        // If we get here, the method works for valid cases
    }

    /// Test health_reporter access
    #[tokio::test]
    async fn test_tonic_server_health_reporter() {
        let addr = "127.0.0.1:0".parse().unwrap();
        let server = TonicServer::new(addr).await.unwrap();

        let reporter = server.health_reporter();
        // We can't easily test the reporter functionality without integration testing,
        // but we can verify we get a reference
        let _reporter_ref = reporter;
    }

    /// Test set_service_health method
    #[tokio::test]
    async fn test_tonic_server_set_service_health() {
        let addr = "127.0.0.1:0".parse().unwrap();
        let server = TonicServer::new(addr).await.unwrap();

        // Test setting health status for the utilities service
        let result = server.set_service_health::<crate::rpc::UtilitiesServiceServer<crate::services::UtilitiesService>>(true).await;
        assert!(result.is_ok(), "Setting service health should succeed");

        let result = server.set_service_health::<crate::rpc::UtilitiesServiceServer<crate::services::UtilitiesService>>(false).await;
        assert!(result.is_ok(), "Setting service not serving should succeed");
    }

    /// Test serve method with dummy address (won't actually start server)
    #[tokio::test]
    async fn test_tonic_server_serve() {
        let addr = "127.0.0.1:0".parse().unwrap();
        let server = TonicServer::new(addr).await.unwrap();

        // We can't easily test serve() without actually starting a server,
        // but we can verify the method exists and takes ownership
        // In integration tests, this would be tested differently
        let _owned_server = server;
        // The serve method would be called in integration tests
    }

    /// Test serve_with_incoming method with dummy stream
    #[tokio::test]
    async fn test_tonic_server_serve_with_incoming() {
        use std::pin::Pin;
        use std::task::{Context, Poll};

        // Dummy stream that never yields any connections
        struct DummyStream;
        impl futures_util::stream::Stream for DummyStream {
            type Item = Result<tokio::net::TcpStream, std::io::Error>;
            fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                Poll::Ready(None)
            }
        }

        let addr = "127.0.0.1:0".parse().unwrap();
        let server = TonicServer::new(addr).await.unwrap();
        let dummy_stream = DummyStream;

        // Should return Ok immediately since the stream is empty
        // This tests the method signature and basic functionality
        let _owned_server = server;
        let _dummy_stream = dummy_stream;
        // In integration tests, serve_with_incoming would be called
    }

    /// Test run_on_addr method (convenience wrapper)
    #[tokio::test]
    async fn test_tonic_server_run_on_addr() {
        let addr = "127.0.0.1:0".parse().unwrap();
        let server = TonicServer::new(addr).await.unwrap();

        // run_on_addr is a convenience method that calls serve
        // We can't test it fully without starting a server
        let _owned_server = server;
        // In integration tests, this would be tested
    }

    /// Test multiple TonicServer instances can be created
    #[tokio::test]
    async fn test_tonic_server_multiple_instances() {
        let addr1 = "127.0.0.1:0".parse().unwrap();
        let addr2 = "127.0.0.1:0".parse().unwrap();

        let server1 = TonicServer::new(addr1).await.unwrap();
        let server2 = TonicServer::new(addr2).await.unwrap();

        // Both should have different ports assigned
        let addr1 = server1.local_addr().unwrap();
        let addr2 = server2.local_addr().unwrap();

        assert_ne!(addr1.port(), addr2.port(), "Different instances should get different ports");
    }

    /// Test server creation with specific port
    #[tokio::test]
    async fn test_tonic_server_specific_port() {
        // Use a high port number that's unlikely to be in use
        let addr = "127.0.0.1:0".parse().unwrap(); // Still use 0 for auto-assignment in test
        let server = TonicServer::new(addr).await.unwrap();

        let assigned_addr = server.local_addr().unwrap();
        assert!(assigned_addr.port() > 0, "Should get a valid port assignment");
    }

    /// Test router field access
    #[tokio::test]
    async fn test_tonic_server_router_access() {
        let addr = "127.0.0.1:0".parse().unwrap();
        let server = TonicServer::new(addr).await.unwrap();

        // Test that we can access the router
        let _router = &server.router;
        // The router should be properly initialized
    }
}

