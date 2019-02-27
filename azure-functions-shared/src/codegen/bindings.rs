mod blob;
mod blob_trigger;
mod event_grid_trigger;
mod event_hub;
mod event_hub_trigger;
mod http;
mod http_trigger;
mod queue;
mod queue_trigger;
mod table;
mod timer_trigger;

pub use self::blob::*;
pub use self::blob_trigger::*;
pub use self::event_grid_trigger::*;
pub use self::event_hub::*;
pub use self::event_hub_trigger::*;
pub use self::http::*;
pub use self::http_trigger::*;
pub use self::queue::*;
pub use self::queue_trigger::*;
pub use self::table::*;
pub use self::timer_trigger::*;

use crate::codegen::{AttributeArguments, TryFrom};
use lazy_static::lazy_static;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::collections::HashMap;

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    In,
    InOut,
    Out,
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
        }
    }

    pub fn binding_type(&self) -> Option<&str> {
        match self {
            Binding::Context => None,
            Binding::HttpTrigger(_) => Some(HTTP_TRIGGER_TYPE),
            Binding::Http(_) => Some(HTTP_TYPE),
            Binding::TimerTrigger(_) => Some(TIMER_TRIGGER_TYPE),
            Binding::QueueTrigger(_) => Some(QUEUE_TRIGGER_TYPE),
            Binding::Queue(_) => Some(QUEUE_TYPE),
            Binding::BlobTrigger(_) => Some(BLOB_TRIGGER_TYPE),
            Binding::Blob(_) => Some(BLOB_TYPE),
            Binding::Table(_) => Some(TABLE_TYPE),
            Binding::EventGridTrigger(_) => Some(EVENT_GRID_TRIGGER_TYPE),
            Binding::EventHubTrigger(_) => Some(EVENT_HUB_TRIGGER_TYPE),
            Binding::EventHub(_) => Some(EVENT_HUB_TYPE),
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
            | Binding::EventHubTrigger(_) => true,
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
        };
    }
}

pub type BindingFactory = fn(AttributeArguments) -> Result<Binding, (Span, String)>;
type BindingMap = HashMap<&'static str, BindingFactory>;

lazy_static! {
    pub static ref TRIGGERS: BindingMap = {
        let mut map: BindingMap = HashMap::new();
        map.insert("HttpRequest", |args| {
            Ok(Binding::HttpTrigger(HttpTrigger::try_from(args)?))
        });
        map.insert("TimerInfo", |args| {
            Ok(Binding::TimerTrigger(TimerTrigger::try_from(args)?))
        });
        map.insert("QueueTrigger", |args| {
            Ok(Binding::QueueTrigger(QueueTrigger::try_from(args)?))
        });
        map.insert("BlobTrigger", |args| {
            Ok(Binding::BlobTrigger(BlobTrigger::try_from(args)?))
        });
        map.insert("EventGridEvent", |args| {
            Ok(Binding::EventGridTrigger(EventGridTrigger::try_from(args)?))
        });
        map.insert("EventHubTrigger", |args| {
            Ok(Binding::EventHubTrigger(EventHubTrigger::try_from(args)?))
        });
        map
    };
    pub static ref INPUT_BINDINGS: BindingMap = {
        let mut map: BindingMap = HashMap::new();
        map.insert("Blob", |args| Ok(Binding::Blob(Blob::try_from(args)?)));
        map.insert("Table", |args| Ok(Binding::Table(Table::try_from(args)?)));
        map
    };
    pub static ref INPUT_OUTPUT_BINDINGS: BindingMap = {
        let mut map: BindingMap = HashMap::new();
        map.insert("BlobTrigger", |args| {
            let mut binding = BlobTrigger::try_from(args)?;
            binding.direction = Direction::InOut;
            Ok(Binding::BlobTrigger(binding))
        });
        map.insert("Blob", |args| {
            let mut binding = Blob::try_from(args)?;
            binding.direction = Direction::InOut;
            Ok(Binding::Blob(binding))
        });
        map
    };
    pub static ref OUTPUT_BINDINGS: BindingMap = {
        let mut map: BindingMap = HashMap::new();
        map.insert("HttpResponse", |args| {
            Ok(Binding::Http(Http::try_from(args)?))
        });
        map.insert("QueueMessage", |args| {
            Ok(Binding::Queue(Queue::try_from(args)?))
        });
        map.insert("Blob", |args| {
            let mut binding = Blob::try_from(args)?;
            binding.direction = Direction::Out;
            Ok(Binding::Blob(binding))
        });
        map.insert("Table", |args| {
            let mut binding = Table::try_from(args)?;
            binding.direction = Direction::Out;
            Ok(Binding::Table(binding))
        });
        map.insert("EventHubMessage", |args| {
            Ok(Binding::EventHub(EventHub::try_from(args)?))
        });
        map
    };
}
