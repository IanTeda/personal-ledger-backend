//-- ./src/main.rs

/// Main server implementation for personal-ledger-backend
/// Implements a gRPC server using Tonic and includes server reflection
/// The server currently implements a simple UtilitiesService with a Ping method to demonstrate functionality.
use tonic::{transport::Server, Request, Response, Status};
use tonic_reflection::server as TonicRefelectionServer;
use personal_ledger_backend::{rpc, telemetry::{self, TelemetryLevel}, LedgerResult};

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

#[tokio::main]
async fn main() -> LedgerResult<()> {

    let _tracing = telemetry::init(TelemetryLevel::INFO);

    // Build reflections service
    let reflections_service = TonicRefelectionServer::Builder::configure()
        .register_encoded_file_descriptor_set(rpc::FILE_DESCRIPTOR_SET)
        .build_v1()
        .unwrap();

    let addr = "0.0.0.0:50051".parse().unwrap();
    let utility_server = MyUtilitiesService::default();

    let (health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<rpc::UtilitiesServiceServer<MyUtilitiesService>>()
        .await;

    tracing::info!("Tonic server started at '{}'", addr);

    // Build Tonic gRPC server
    Server::builder()
        .add_service(health_service)
        .add_service(reflections_service)
        .add_service(rpc::UtilitiesServiceServer::new(utility_server))
        .serve(addr)
        .await?;

    Ok(())
}