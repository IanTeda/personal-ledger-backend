/// Type alias for the categories service client to improve readability.
pub type CategoryServicesClient = personal_ledger_backend::rpc::CategoriesServiceClient<tonic::transport::Channel>;

/// A test gRPC client for integration testing.
///
/// This struct provides a convenient wrapper around gRPC service clients
/// for use in integration tests. It manages client connections and provides
/// access to individual service clients.
///
/// Currently supports the Categories service, with potential for expansion
/// to other services as the API grows.
#[derive(Clone)]
pub struct SpawnTonicClient {
    /// The categories service client for making RPC calls
    category: CategoryServicesClient,
}

impl SpawnTonicClient {
    /// Creates a new test gRPC client with the provided transport channel.
    ///
    /// # Arguments
    ///
    /// * `tonic_channel` - The transport channel to use for gRPC communication
    ///
    /// # Returns
    ///
    /// A new `SpawnTonicClient` instance ready for making RPC calls
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = SpawnTonicClient::new(channel);
    /// ```
    pub fn new(tonic_channel: tonic::transport::Channel) -> Self {
        let category = CategoryServicesClient::new(tonic_channel);
        Self { category }
    }

    /// Returns a mutable reference to the categories service client.
    ///
    /// This allows making RPC calls to the categories service endpoints.
    /// The mutable reference is required by tonic for making requests.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `CategoryServicesClient`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut client = SpawnTonicClient::new(channel);
    /// let response = client.category().category_create(request).await?;
    /// ```
    pub fn category(&mut self) -> &mut CategoryServicesClient {
        &mut self.category
    }
}