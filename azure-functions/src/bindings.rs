//! Module for Azure Functions bindings.
mod blob;
mod blob_trigger;
mod cosmos_db_documents;
mod cosmos_db_trigger;
mod event_grid_event;
mod event_hub_message;
mod event_hub_trigger;
mod http_request;
mod http_response;
mod queue_message;
mod queue_trigger;
mod table;
mod timer_info;

pub use self::blob::*;
pub use self::blob_trigger::*;
pub use self::cosmos_db_documents::*;
pub use self::cosmos_db_trigger::*;
pub use self::event_grid_event::*;
pub use self::event_hub_message::*;
pub use self::event_hub_trigger::*;
pub use self::http_request::*;
pub use self::http_response::*;
pub use self::queue_message::*;
pub use self::queue_trigger::*;
pub use self::table::*;
pub use self::timer_info::*;
