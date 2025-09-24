use config::ConfigError as ConfigLibError;

#[derive(thiserror::Error, Debug)]
/// Errors produced while loading or validating configuration.
///
/// This enum wraps errors from the underlying `config` crate and adds
/// domain-specific validation variants such as an invalid server address.
pub enum ConfigError {
    /// Error from the underlying config crate during file loading or parsing.
    ///
    /// This wraps errors from the `config` crate such as file not found,
    /// invalid syntax, or parsing failures.
    #[error("Configuration parsing error: {0}")]
    Parsing(#[from] ConfigLibError),

    /// Validation error with a descriptive message.
    ///
    /// This is used for configuration validation failures such as invalid
    /// values, missing required fields, or security misconfigurations.
    #[error("Invalid configuration: {0}")]
    Validation(String),

    /// Error indicating an invalid server address format.
    ///
    /// This is used for configuration validation failures such as invalid
    /// values, missing required fields, or security misconfigurations.
    #[error("Invalid server address: {0}")]
    InvalidServerAddress(#[from] std::net::AddrParseError),
}