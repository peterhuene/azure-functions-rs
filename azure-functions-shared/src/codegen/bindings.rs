mod activity_trigger;
mod blob;
mod blob_trigger;
mod cosmos_db;
mod cosmos_db_trigger;
mod durable_client;
mod entity_trigger;
mod event_grid_trigger;
mod event_hub;
mod event_hub_trigger;
mod generic;
mod http;
mod http_trigger;
mod orchestration_trigger;
mod queue;
mod queue_trigger;
mod send_grid;
mod service_bus;
mod service_bus_trigger;
mod signalr;
mod signalr_connection_info;
mod table;
mod timer_trigger;
mod twilio_sms;

pub use self::activity_trigger::*;
pub use self::blob::*;
pub use self::blob_trigger::*;
pub use self::cosmos_db::*;
pub use self::cosmos_db_trigger::*;
pub use self::durable_client::*;
pub use self::entity_trigger::*;
pub use self::event_grid_trigger::*;
pub use self::event_hub::*;
pub use self::event_hub_trigger::*;
pub use self::generic::*;
pub use self::http::*;
pub use self::http_trigger::*;
pub use self::orchestration_trigger::*;
pub use self::queue::*;
pub use self::queue_trigger::*;
pub use self::send_grid::*;
pub use self::service_bus::*;
pub use self::service_bus_trigger::*;
pub use self::signalr::*;
pub use self::signalr_connection_info::*;
pub use self::table::*;
pub use self::timer_trigger::*;
pub use self::twilio_sms::*;

use lazy_static::lazy_static;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
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
        Self::In
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(untagged, rename_all = "camelCase")]
#[allow(clippy::large_enum_variant)]
pub enum Binding {
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
    ServiceBusTrigger(ServiceBusTrigger),
    ServiceBus(ServiceBus),
    TwilioSms(TwilioSms),
    SendGrid(SendGrid),
    GenericTrigger(Generic),
    Generic(Generic),
    DurableClient(DurableClient),
    OrchestrationTrigger(OrchestrationTrigger),
    ActivityTrigger(ActivityTrigger),
    EntityTrigger(EntityTrigger),
}

impl Binding {
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::HttpTrigger(b) => Some(&b.name),
            Self::Http(b) => Some(&b.name),
            Self::TimerTrigger(b) => Some(&b.name),
            Self::QueueTrigger(b) => Some(&b.name),
            Self::Queue(b) => Some(&b.name),
            Self::BlobTrigger(b) => Some(&b.name),
            Self::Blob(b) => Some(&b.name),
            Self::Table(b) => Some(&b.name),
            Self::EventGridTrigger(b) => Some(&b.name),
            Self::EventHubTrigger(b) => Some(&b.name),
            Self::EventHub(b) => Some(&b.name),
            Self::CosmosDbTrigger(b) => Some(&b.name),
            Self::CosmosDb(b) => Some(&b.name),
            Self::SignalRConnectionInfo(b) => Some(&b.name),
            Self::SignalR(b) => Some(&b.name),
            Self::ServiceBusTrigger(b) => Some(&b.name),
            Self::ServiceBus(b) => Some(&b.name),
            Self::TwilioSms(b) => Some(&b.name),
            Self::SendGrid(b) => Some(&b.name),
            Self::GenericTrigger(b) => Some(&b.name),
            Self::Generic(b) => Some(&b.name),
            Self::DurableClient(b) => Some(&b.name),
            Self::OrchestrationTrigger(b) => Some(&b.name),
            Self::ActivityTrigger(b) => Some(&b.name),
            Self::EntityTrigger(b) => Some(&b.name),
        }
    }

    pub fn binding_type(&self) -> Option<&str> {
        match self {
            Self::HttpTrigger(_) => Some(HttpTrigger::binding_type()),
            Self::Http(_) => Some(HttpTrigger::binding_type()),
            Self::TimerTrigger(_) => Some(TimerTrigger::binding_type()),
            Self::QueueTrigger(_) => Some(QueueTrigger::binding_type()),
            Self::Queue(_) => Some(Queue::binding_type()),
            Self::BlobTrigger(_) => Some(BlobTrigger::binding_type()),
            Self::Blob(_) => Some(Blob::binding_type()),
            Self::Table(_) => Some(Table::binding_type()),
            Self::EventGridTrigger(_) => Some(EventGridTrigger::binding_type()),
            Self::EventHubTrigger(_) => Some(EventHubTrigger::binding_type()),
            Self::EventHub(_) => Some(EventHub::binding_type()),
            Self::CosmosDbTrigger(_) => Some(CosmosDbTrigger::binding_type()),
            Self::CosmosDb(_) => Some(CosmosDb::binding_type()),
            Self::SignalRConnectionInfo(_) => Some(SignalRConnectionInfo::binding_type()),
            Self::SignalR(_) => Some(SignalR::binding_type()),
            Self::ServiceBusTrigger(_) => Some(ServiceBusTrigger::binding_type()),
            Self::ServiceBus(_) => Some(ServiceBus::binding_type()),
            Self::TwilioSms(_) => Some(TwilioSms::binding_type()),
            Self::SendGrid(_) => Some(SendGrid::binding_type()),
            Self::GenericTrigger(b) => Some(b.binding_type()),
            Self::Generic(b) => Some(b.binding_type()),
            Self::DurableClient(_) => Some(DurableClient::binding_type()),
            Self::OrchestrationTrigger(_) => Some(OrchestrationTrigger::binding_type()),
            Self::ActivityTrigger(_) => Some(ActivityTrigger::binding_type()),
            Self::EntityTrigger(_) => Some(EntityTrigger::binding_type()),
        }
    }

    pub fn is_trigger(&self) -> bool {
        match self {
            Self::HttpTrigger(_)
            | Self::TimerTrigger(_)
            | Self::QueueTrigger(_)
            | Self::BlobTrigger(_)
            | Self::EventGridTrigger(_)
            | Self::EventHubTrigger(_)
            | Self::CosmosDbTrigger(_)
            | Self::ServiceBusTrigger(_)
            | Self::GenericTrigger(_)
            | Self::OrchestrationTrigger(_)
            | Self::ActivityTrigger(_)
            | Self::EntityTrigger(_) => true,
            _ => false,
        }
    }
}

impl ToTokens for Binding {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::HttpTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::HttpTrigger(#b))
            }
            Self::Http(b) => quote!(::azure_functions::codegen::bindings::Binding::Http(#b)),
            Self::TimerTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::TimerTrigger(#b))
            }
            Self::QueueTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::QueueTrigger(#b))
            }
            Self::Queue(b) => quote!(::azure_functions::codegen::bindings::Binding::Queue(#b)),
            Self::BlobTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::BlobTrigger(#b))
            }
            Self::Blob(b) => quote!(::azure_functions::codegen::bindings::Binding::Blob(#b)),
            Self::Table(b) => quote!(::azure_functions::codegen::bindings::Binding::Table(#b)),
            Self::EventGridTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::EventGridTrigger(#b))
            }
            Self::EventHubTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::EventHubTrigger(#b))
            }
            Self::EventHub(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::EventHub(#b))
            }
            Self::CosmosDbTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::CosmosDbTrigger(#b))
            }
            Self::CosmosDb(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::CosmosDb(#b))
            }
            Self::SignalRConnectionInfo(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::SignalRConnectionInfo(#b))
            }
            Self::SignalR(b) => quote!(::azure_functions::codegen::bindings::Binding::SignalR(#b)),
            Self::ServiceBusTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::ServiceBusTrigger(#b))
            }
            Self::ServiceBus(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::ServiceBus(#b))
            }
            Self::TwilioSms(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::TwilioSms(#b))
            }
            Self::SendGrid(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::SendGrid(#b))
            }
            Self::GenericTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::GenericTrigger(#b))
            }
            Self::Generic(b) => quote!(::azure_functions::codegen::bindings::Binding::Generic(#b)),
            Self::DurableClient(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::DurableClient(#b))
            }
            Self::OrchestrationTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::OrchestrationTrigger(#b))
            }
            Self::ActivityTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::ActivityTrigger(#b))
            }
            Self::EntityTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::EntityTrigger(#b))
            }
        }
        .to_tokens(tokens);
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
        map.insert("ServiceBusTrigger", |args, span| {
            Binding::ServiceBusTrigger(ServiceBusTrigger::from((args, span)))
        });
        map.insert("GenericTrigger", |args, span| {
            Binding::GenericTrigger(Generic::from((args, span)))
        });
        map.insert("DurableOrchestrationContext", |args, span| {
            Binding::OrchestrationTrigger(OrchestrationTrigger::from((args, span)))
        });
        map.insert("DurableActivityContext", |args, span| {
            Binding::ActivityTrigger(ActivityTrigger::from((args, span)))
        });
        map.insert("DurableEntityContext", |args, span| {
            Binding::EntityTrigger(EntityTrigger::from((args, span)))
        });
        map
    };
    pub static ref INPUT_BINDINGS: BindingMap = {
        let mut map: BindingMap = HashMap::new();
        map.insert("Blob", |args, span| Binding::Blob(Blob::from((args, span))));
        map.insert("Table", |args, span| {
            Binding::Table(Table::from((args, span)))
        });
        map.insert("CosmosDbDocument", |args, span| {
            Binding::CosmosDb(CosmosDb::from((args, span)))
        });
        map.insert("SignalRConnectionInfo", |args, span| {
            Binding::SignalRConnectionInfo(SignalRConnectionInfo::from((args, span)))
        });
        map.insert("GenericInput", |args, span| {
            Binding::Generic(Generic::from((args, span)))
        });
        map.insert("DurableOrchestrationClient", |args, span| {
            Binding::DurableClient(DurableClient::from((args, span)))
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
        map.insert("CosmosDbDocument", |args, span| {
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
        map.insert("CosmosDbDocument", |args, span| {
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
        map.insert("ServiceBusMessage", |args, span| {
            Binding::ServiceBus(ServiceBus::from((args, span)))
        });
        map.insert("TwilioSmsMessage", |args, span| {
            Binding::TwilioSms(TwilioSms::from((args, span)))
        });
        map.insert("SendGridMessage", |args, span| {
            Binding::SendGrid(SendGrid::from((args, span)))
        });
        map.insert("GenericOutput", |args, span| {
            let mut binding = Generic::from((args, span));
            binding.direction = Direction::Out;
            Binding::Generic(binding)
        });
        map
    };
    pub static ref VEC_INPUT_BINDINGS: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert("CosmosDbDocument");
        set
    };
    pub static ref VEC_OUTPUT_BINDINGS: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert("CosmosDbDocument");
        set.insert("EventHubMessage");
        set.insert("QueueMessage");
        set.insert("SignalRMessage");
        set.insert("SignalRGroupAction");
        set.insert("ServiceBusMessage");
        set.insert("TwilioSmsMessage");
        set.insert("SendGridMessage");
        set
    };
}
