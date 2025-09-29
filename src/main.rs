//-- ./src/main.rs

use tonic::{transport::Server, Request, Response, Status};
use tonic_reflection::server as TonicRefelectionServer;
use personal_ledger_backend::{rpc, telemetry, LedgerResult};

#[derive(Default)]
pub struct MyUtilitiesService {}

#[tonic::async_trait]
impl rpc::UtilitiesService for MyUtilitiesService {
    async fn ping(
        &self,
        request: Request<rpc::PingRequest>,
    ) -> Result<Response<rpc::PingResponse>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply: rpc::PingResponse = rpc::PingResponse {
            message: "Pong...".to_string(),
        };

        Ok(Response::new(reply)) // Send back ping response
    }
}

/// Main server implementation for personal-ledger-backend
/// Implements a gRPC server using Tonic and includes server reflection
/// The server currently implements a simple UtilitiesService with a Ping method to demonstrate functionality.
#[tokio::main]
async fn main() -> LedgerResult<()> {

    let config = personal_ledger_backend::LedgerConfig::parse()?;
    let log_level = config.server.log_level();

    let _tracing = telemetry::init(log_level);
    tracing::info!("Starting tracing at level '{:?}'", log_level);

    // Initialize the database connection pool and run migrations
    let _db_pool = personal_ledger_backend::database::connect(&config).await?;
    tracing::info!("Database initialized and migrations run");

    // Build reflections service
    let reflections_service = TonicRefelectionServer::Builder::configure()
        .register_encoded_file_descriptor_set(rpc::FILE_DESCRIPTOR_SET)
        .build_v1()
        .unwrap();

    let server_address = config.server.address()?;
    tracing::info!("Starting Tonic server at '{}'", server_address);

    let utility_server = MyUtilitiesService::default();

    let (health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<rpc::UtilitiesServiceServer<MyUtilitiesService>>()
        .await;

    tracing::info!("Tonic server started at '{}'", server_address);

    // Build Tonic gRPC server
    Server::builder()
        .add_service(health_service)
        .add_service(reflections_service)
        .add_service(rpc::UtilitiesServiceServer::new(utility_server))
        .serve(server_address)
        .await?;

    Ok(())
}