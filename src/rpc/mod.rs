// mod.rs
// RPC module for personal-ledger-backend
// Contains reexports for gRPC services, clients, servers, and protobuf types

pub mod personal_ledger;

pub mod utilities {
    pub use super::personal_ledger::utilities_service_client::UtilitiesServiceClient as client;
    pub use super::personal_ledger::utilities_service_server::UtilitiesServiceServer as server;
    pub use super::personal_ledger::PingRequest as ping_request;
    pub use super::personal_ledger::PingResponse as ping_response;

}

// Reexports for easy access from main.rs
pub use personal_ledger::utilities_service_server::{UtilitiesService, UtilitiesServiceServer};
pub use personal_ledger::utilities_service_client::UtilitiesServiceClient;
pub use personal_ledger::{PingRequest, PingResponse};
