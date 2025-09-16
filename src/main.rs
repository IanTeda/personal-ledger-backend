//-- ./src/main.rs

/// Main server implementation for personal-ledger-backend
/// Implements a gRPC server using Tonic and includes server reflection
/// The server currently implements a simple UtilitiesService with a Ping method to demonstrate functionality.
use tonic::{transport::Server, Request, Response, Status};
use tonic_reflection::server as TonicRefelectionServer;

mod proto {
    // The string specified here must match the proto package name
    tonic::include_proto!("personal_ledger");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("personal_ledger_descriptor");
}

use crate::proto::utilities_service_server::UtilitiesService;
use crate::proto::utilities_service_server::UtilitiesServiceServer;
use crate::proto::{PingRequest, PingResponse};

#[derive(Default)]
pub struct MyUtilitiesService {}

#[tonic::async_trait]
impl UtilitiesService for MyUtilitiesService {
    async fn ping(
        &self,
        request: Request<PingRequest>,
    ) -> Result<Response<PingResponse>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply: PingResponse = PingResponse {
            message: "Pong...".to_string(),
        };

        Ok(Response::new(reply)) // Send back ping response
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let reflections_service = TonicRefelectionServer::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build_v1()
        .unwrap();

    let addr = "0.0.0.0:50051".parse().unwrap();
    let utility_server = MyUtilitiesService::default();

    println!("UtilitiesServiceServer listening on {addr}");

    Server::builder()
        .add_service(reflections_service)
        .add_service(UtilitiesServiceServer::new(utility_server))
        .serve(addr)
        .await?;

    Ok(())
}