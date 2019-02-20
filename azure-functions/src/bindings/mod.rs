//! Module for Azure Functions bindings.

use crate::rpc::protocol;
use std::collections::HashMap;

mod blob;
mod blob_trigger;
mod event_grid_event;
mod http_request;
mod http_response;
mod queue_message;
mod queue_trigger;
mod table;
mod timer_info;

pub use self::blob::*;
pub use self::blob_trigger::*;
pub use self::event_grid_event::*;
pub use self::http_request::*;
pub use self::http_response::*;
pub use self::queue_message::*;
pub use self::queue_trigger::*;
pub use self::table::*;
pub use self::timer_info::*;

#[doc(hidden)]
pub trait Trigger {
    fn read_metadata(&mut self, metadata: &mut HashMap<String, protocol::TypedData>);
}
