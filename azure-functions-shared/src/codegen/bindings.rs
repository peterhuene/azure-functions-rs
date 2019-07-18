mod activity_trigger;
mod blob;
mod blob_trigger;
mod cosmos_db;
mod cosmos_db_trigger;
mod event_grid_trigger;
mod event_hub;
mod event_hub_trigger;
mod generic;
mod http;
mod http_trigger;
mod orchestration_client;
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
pub use self::event_grid_trigger::*;
pub use self::event_hub::*;
pub use self::event_hub_trigger::*;
pub use self::generic::*;
pub use self::http::*;
pub use self::http_trigger::*;
pub use self::orchestration_client::*;
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
use serde_derive::Serialize;
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
        Direction::In
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
    OrchestrationClient(OrchestrationClient),
    OrchestrationTrigger(OrchestrationTrigger),
    ActivityTrigger(ActivityTrigger),
}

impl Binding {
    pub fn name(&self) -> Option<&str> {
        match self {
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
            Binding::ServiceBusTrigger(b) => Some(&b.name),
            Binding::ServiceBus(b) => Some(&b.name),
            Binding::TwilioSms(b) => Some(&b.name),
            Binding::SendGrid(b) => Some(&b.name),
            Binding::GenericTrigger(b) => Some(&b.name),
            Binding::Generic(b) => Some(&b.name),
            Binding::OrchestrationClient(b) => Some(&b.name),
            Binding::OrchestrationTrigger(b) => Some(&b.name),
            Binding::ActivityTrigger(b) => Some(&b.name),
        }
    }

    pub fn binding_type(&self) -> Option<&str> {
        match self {
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
            Binding::ServiceBusTrigger(_) => Some(ServiceBusTrigger::binding_type()),
            Binding::ServiceBus(_) => Some(ServiceBus::binding_type()),
            Binding::TwilioSms(_) => Some(TwilioSms::binding_type()),
            Binding::SendGrid(_) => Some(SendGrid::binding_type()),
            Binding::GenericTrigger(b) => Some(b.binding_type()),
            Binding::Generic(b) => Some(b.binding_type()),
            Binding::OrchestrationClient(_) => Some(OrchestrationClient::binding_type()),
            Binding::OrchestrationTrigger(_) => Some(OrchestrationTrigger::binding_type()),
            Binding::ActivityTrigger(_) => Some(ActivityTrigger::binding_type()),
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
            | Binding::CosmosDbTrigger(_)
            | Binding::ServiceBusTrigger(_)
            | Binding::GenericTrigger(_)
            | Binding::OrchestrationTrigger(_)
            | Binding::ActivityTrigger(_) => true,
            _ => false,
        }
    }
}

impl ToTokens for Binding {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Binding::HttpTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::HttpTrigger(#b))
            }
            Binding::Http(b) => quote!(::azure_functions::codegen::bindings::Binding::Http(#b)),
            Binding::TimerTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::TimerTrigger(#b))
            }
            Binding::QueueTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::QueueTrigger(#b))
            }
            Binding::Queue(b) => quote!(::azure_functions::codegen::bindings::Binding::Queue(#b)),
            Binding::BlobTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::BlobTrigger(#b))
            }
            Binding::Blob(b) => quote!(::azure_functions::codegen::bindings::Binding::Blob(#b)),
            Binding::Table(b) => quote!(::azure_functions::codegen::bindings::Binding::Table(#b)),
            Binding::EventGridTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::EventGridTrigger(#b))
            }
            Binding::EventHubTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::EventHubTrigger(#b))
            }
            Binding::EventHub(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::EventHub(#b))
            }
            Binding::CosmosDbTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::CosmosDbTrigger(#b))
            }
            Binding::CosmosDb(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::CosmosDb(#b))
            }
            Binding::SignalRConnectionInfo(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::SignalRConnectionInfo(#b))
            }
            Binding::SignalR(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::SignalR(#b))
            }
            Binding::ServiceBusTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::ServiceBusTrigger(#b))
            }
            Binding::ServiceBus(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::ServiceBus(#b))
            }
            Binding::TwilioSms(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::TwilioSms(#b))
            }
            Binding::SendGrid(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::SendGrid(#b))
            }
            Binding::GenericTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::GenericTrigger(#b))
            }
            Binding::Generic(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::Generic(#b))
            }
            Binding::OrchestrationClient(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::OrchestrationClient(#b))
            }
            Binding::OrchestrationTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::OrchestrationTrigger(#b))
            }
            Binding::ActivityTrigger(b) => {
                quote!(::azure_functions::codegen::bindings::Binding::ActivityTrigger(#b))
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
            Binding::OrchestrationClient(OrchestrationClient::from((args, span)))
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
