use tonic::{transport::Server, Request, Response, Status};

use personal_ledger_backend::rpc::{UtilitiesService, UtilitiesServiceServer, PingRequest, PingResponse};

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
    let addr = "0.0.0.0:50051".parse().unwrap();
    let utility_server = MyUtilitiesService::default();

    println!("UtilitiesServiceServer listening on {addr}");

    Server::builder()
        .add_service(UtilitiesServiceServer::new(utility_server))
        .serve(addr)
        .await?;

    Ok(())
}