//! # Durable Functions HTTP client for Rust.

#![deny(missing_docs)]

mod client;
mod endpoint;
mod error;

pub use self::client::*;
pub use self::endpoint::*;
pub use self::error::*;

/// The result type for the Durable Functions HTTP client.
pub type Result<T> = std::result::Result<T, ClientError>;
