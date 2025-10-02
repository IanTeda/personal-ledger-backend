mod error;
pub use error::DatabaseError;
pub use error::DatabaseResult;

mod connect;
pub use connect::connect;

mod categories;
pub use categories::*;