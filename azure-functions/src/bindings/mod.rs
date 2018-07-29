//! Module for Azure Functions bindings.

use rpc::protocol;
use std::collections::HashMap;

mod blob_trigger;
mod http_request;
mod http_response;
mod queue_message;
mod queue_trigger;
mod timer_info;

pub use self::blob_trigger::*;
pub use self::http_request::*;
pub use self::http_response::*;
pub use self::queue_message::*;
pub use self::queue_trigger::*;
pub use self::timer_info::*;

#[doc(hidden)]
pub trait Trigger<'a> {
    fn read_metadata(&mut self, metadata: &'a HashMap<String, protocol::TypedData>);
}
