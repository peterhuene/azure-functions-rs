mod blob;
mod blob_trigger;
mod cosmos_db;
mod cosmos_db_trigger;
mod event_grid_trigger;
mod event_hub;
mod event_hub_trigger;
mod http;
mod http_trigger;
mod queue;
mod queue_trigger;
mod signalr;
mod signalr_connection_info;
mod table;
mod timer_trigger;

pub use self::blob::*;
pub use self::blob_trigger::*;
pub use self::cosmos_db::*;
pub use self::cosmos_db_trigger::*;
pub use self::event_grid_trigger::*;
pub use self::event_hub::*;
pub use self::event_hub_trigger::*;
pub use self::http::*;
pub use self::http_trigger::*;
pub use self::queue::*;
pub use self::queue_trigger::*;
pub use self::signalr::*;
pub use self::signalr_connection_info::*;
pub use self::table::*;
pub use self::timer_trigger::*;

use lazy_static::lazy_static;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::collections::HashMap;
use syn::AttributeArgs;

#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    In,
    InOut,
    Out,
}

impl Default for Direction {
    fn default() -> Self {
        Direction::In
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(untagged, rename_all = "camelCase")]
#[allow(clippy::large_enum_variant)]
pub enum Binding {
    Context,
    HttpTrigger(HttpTrigger),
    Http(Http),
    TimerTrigger(TimerTrigger),
    QueueTrigger(QueueTrigger),
    Queue(Queue),
    BlobTrigger(BlobTrigger),
    Blob(Blob),
    Table(Table),
    EventGridTrigger(EventGridTrigger),
    EventHubTrigger(EventHubTrigger),
    EventHub(EventHub),
    CosmosDbTrigger(CosmosDbTrigger),
    CosmosDb(CosmosDb),
    SignalRConnectionInfo(SignalRConnectionInfo),
    SignalR(SignalR),
}

impl Binding {
    pub fn name(&self) -> Option<&str> {
        match self {
            Binding::Context => None,
            Binding::HttpTrigger(b) => Some(&b.name),
            Binding::Http(b) => Some(&b.name),
            Binding::TimerTrigger(b) => Some(&b.name),
            Binding::QueueTrigger(b) => Some(&b.name),
            Binding::Queue(b) => Some(&b.name),
            Binding::BlobTrigger(b) => Some(&b.name),
            Binding::Blob(b) => Some(&b.name),
            Binding::Table(b) => Some(&b.name),
            Binding::EventGridTrigger(b) => Some(&b.name),
            Binding::EventHubTrigger(b) => Some(&b.name),
            Binding::EventHub(b) => Some(&b.name),
            Binding::CosmosDbTrigger(b) => Some(&b.name),
            Binding::CosmosDb(b) => Some(&b.name),
            Binding::SignalRConnectionInfo(b) => Some(&b.name),
            Binding::SignalR(b) => Some(&b.name),
        }
    }

    pub fn binding_type(&self) -> Option<&str> {
        match self {
            Binding::Context => None,
            Binding::HttpTrigger(_) => Some(HttpTrigger::binding_type()),
            Binding::Http(_) => Some(HttpTrigger::binding_type()),
            Binding::TimerTrigger(_) => Some(TimerTrigger::binding_type()),
            Binding::QueueTrigger(_) => Some(QueueTrigger::binding_type()),
            Binding::Queue(_) => Some(Queue::binding_type()),
            Binding::BlobTrigger(_) => Some(BlobTrigger::binding_type()),
            Binding::Blob(_) => Some(Blob::binding_type()),
            Binding::Table(_) => Some(Table::binding_type()),
            Binding::EventGridTrigger(_) => Some(EventGridTrigger::binding_type()),
            Binding::EventHubTrigger(_) => Some(EventHubTrigger::binding_type()),
            Binding::EventHub(_) => Some(EventHub::binding_type()),
            Binding::CosmosDbTrigger(_) => Some(CosmosDbTrigger::binding_type()),
            Binding::CosmosDb(_) => Some(CosmosDb::binding_type()),
            Binding::SignalRConnectionInfo(_) => Some(SignalRConnectionInfo::binding_type()),
            Binding::SignalR(_) => Some(SignalR::binding_type()),
        }
    }

    pub fn is_context(&self) -> bool {
        match self {
            Binding::Context => true,
            _ => false,
        }
    }

    pub fn is_trigger(&self) -> bool {
        match self {
            Binding::HttpTrigger(_)
            | Binding::TimerTrigger(_)
            | Binding::QueueTrigger(_)
            | Binding::BlobTrigger(_)
            | Binding::EventGridTrigger(_)
            | Binding::EventHubTrigger(_)
            | Binding::CosmosDbTrigger(_) => true,
            _ => false,
        }
    }
}

impl ToTokens for Binding {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Binding::Context => panic!("context bindings cannot be tokenized"),
            Binding::HttpTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::HttpTrigger(#b))
                    .to_tokens(tokens)
            }
            Binding::Http(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::Http(#b)).to_tokens(tokens)
            }
            Binding::TimerTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::TimerTrigger(#b))
                    .to_tokens(tokens)
            }
            Binding::QueueTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::QueueTrigger(#b))
                    .to_tokens(tokens)
            }
            Binding::Queue(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::Queue(#b)).to_tokens(tokens)
            }
            Binding::BlobTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::BlobTrigger(#b))
                    .to_tokens(tokens)
            }
            Binding::Blob(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::Blob(#b)).to_tokens(tokens)
            }
            Binding::Table(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::Table(#b)).to_tokens(tokens)
            }
            Binding::EventGridTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::EventGridTrigger(#b))
                    .to_tokens(tokens)
            }
            Binding::EventHubTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::EventHubTrigger(#b))
                    .to_tokens(tokens)
            }
            Binding::EventHub(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::EventHub(#b))
                    .to_tokens(tokens)
            }
            Binding::CosmosDbTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::CosmosDbTrigger(#b))
                    .to_tokens(tokens)
            }
            Binding::CosmosDb(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::CosmosDb(#b))
                    .to_tokens(tokens)
            }
            Binding::SignalRConnectionInfo(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::SignalRConnectionInfo(#b))
                    .to_tokens(tokens)
            }
            Binding::SignalR(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::SignalR(#b)).to_tokens(tokens)
            }
        };
    }
}

pub type BindingFactory = fn(AttributeArgs, Span) -> Binding;
type BindingMap = HashMap<&'static str, BindingFactory>;

lazy_static! {
    pub static ref TRIGGERS: BindingMap = {
        let mut map: BindingMap = HashMap::new();
        map.insert("HttpRequest", |args, span| {
            Binding::HttpTrigger(HttpTrigger::from((args, span)))
        });
        map.insert("TimerInfo", |args, span| {
            Binding::TimerTrigger(TimerTrigger::from((args, span)))
        });
        map.insert("QueueTrigger", |args, span| {
            Binding::QueueTrigger(QueueTrigger::from((args, span)))
        });
        map.insert("BlobTrigger", |args, span| {
            Binding::BlobTrigger(BlobTrigger::from((args, span)))
        });
        map.insert("EventGridEvent", |args, span| {
            Binding::EventGridTrigger(EventGridTrigger::from((args, span)))
        });
        map.insert("EventHubTrigger", |args, span| {
            Binding::EventHubTrigger(EventHubTrigger::from((args, span)))
        });
        map.insert("CosmosDbTrigger", |args, span| {
            Binding::CosmosDbTrigger(CosmosDbTrigger::from((args, span)))
        });
        map
    };
    pub static ref INPUT_BINDINGS: BindingMap = {
        let mut map: BindingMap = HashMap::new();
        map.insert("Blob", |args, span| Binding::Blob(Blob::from((args, span))));
        map.insert("Table", |args, span| {
            Binding::Table(Table::from((args, span)))
        });
        map.insert("CosmosDbDocuments", |args, span| {
            Binding::CosmosDb(CosmosDb::from((args, span)))
        });
        map.insert("SignalRConnectionInfo", |args, span| {
            Binding::SignalRConnectionInfo(SignalRConnectionInfo::from((args, span)))
        });
        map
    };
    pub static ref INPUT_OUTPUT_BINDINGS: BindingMap = {
        let mut map: BindingMap = HashMap::new();
        map.insert("BlobTrigger", |args, span| {
            let mut binding = BlobTrigger::from((args, span));
            binding.direction = Direction::InOut;
            Binding::BlobTrigger(binding)
        });
        map.insert("Blob", |args, span| {
            let mut binding = Blob::from((args, span));
            binding.direction = Direction::InOut;
            Binding::Blob(binding)
        });
        map.insert("CosmosDbDocuments", |args, span| {
            let mut binding = CosmosDb::from((args, span));
            binding.direction = Direction::InOut;
            Binding::CosmosDb(binding)
        });
        map
    };
    pub static ref OUTPUT_BINDINGS: BindingMap = {
        let mut map: BindingMap = HashMap::new();
        map.insert("HttpResponse", |args, span| {
            Binding::Http(Http::from((args, span)))
        });
        map.insert("QueueMessage", |args, span| {
            Binding::Queue(Queue::from((args, span)))
        });
        map.insert("Blob", |args, span| {
            let mut binding = Blob::from((args, span));
            binding.direction = Direction::Out;
            Binding::Blob(binding)
        });
        map.insert("Table", |args, span| {
            let mut binding = Table::from((args, span));
            binding.direction = Direction::Out;
            Binding::Table(binding)
        });
        map.insert("EventHubMessage", |args, span| {
            Binding::EventHub(EventHub::from((args, span)))
        });
        map.insert("CosmosDbDocuments", |args, span| {
            let mut binding = CosmosDb::from((args, span));
            binding.direction = Direction::Out;
            Binding::CosmosDb(binding)
        });
        map.insert("SignalRMessage", |args, span| {
            Binding::SignalR(SignalR::from((args, span)))
        });
        map.insert("SignalRGroupAction", |args, span| {
            Binding::SignalR(SignalR::from((args, span)))
        });
        map
    };
}

#[cfg(test)]
mod tests {
    use std::panic::{catch_unwind, UnwindSafe};

    pub fn should_panic<T>(callback: T, msg: &str)
    where
        T: FnOnce() + UnwindSafe,
    {
        let result = catch_unwind(|| callback());
        assert!(result.is_err(), "the function did not panic");

        if cfg!(feature = "unstable") {
            assert_eq!(
                result.unwrap_err().downcast_ref::<String>().unwrap(),
                "aborting due to previous error",
                "the panic message is not the expected one"
            );
        } else {
            assert_eq!(
                result.unwrap_err().downcast_ref::<String>().unwrap(),
                msg,
                "the panic message is not the expected one"
            );
        }
    }
}
