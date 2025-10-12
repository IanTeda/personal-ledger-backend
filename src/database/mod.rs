mod error;
pub use error::DatabaseError;
pub use error::DatabaseResult;

mod connect;
pub use connect::connect;

#[cfg(test)]
pub use connect::{connect_test, drop_test_database};

#[cfg(test)]
pub mod test;

mod categories;
pub use categories::Category;
