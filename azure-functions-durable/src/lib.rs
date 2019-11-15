//! # Durable Functions HTTP client for Rust.

#![deny(missing_docs)]
#![deny(unused_extern_crates)]
#![warn(clippy::use_self)]
#![warn(clippy::option_map_unwrap_or)]
#![warn(clippy::option_map_unwrap_or_else)]

mod client;
mod endpoint;
mod error;

pub use self::client::*;
pub use self::endpoint::*;
pub use self::error::*;

/// The result type for the Durable Functions HTTP client.
pub type Result<T> = std::result::Result<T, ClientError>;
