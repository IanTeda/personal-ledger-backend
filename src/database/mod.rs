mod error;
pub use error::DatabaseError;
pub use error::DatabaseResult;

mod connect;
pub use connect::connect;

pub mod categories;