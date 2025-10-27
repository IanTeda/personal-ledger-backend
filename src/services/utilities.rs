use crate::rpc;

#[derive(Default)]
pub struct UtilitiesService {}

#[tonic::async_trait]
impl rpc::UtilitiesService for UtilitiesService {
    async fn ping(
        &self,
        request: tonic::Request<rpc::PingRequest>,
    ) -> Result<tonic::Response<rpc::PingResponse>, tonic::Status> {
        tracing::debug!("Got a request from {:?}", request.remote_addr());

        let reply: rpc::PingResponse = rpc::PingResponse {
            message: "Pong...".to_string(),
        };

        Ok(tonic::Response::new(reply)) // Send back ping response
    }
}