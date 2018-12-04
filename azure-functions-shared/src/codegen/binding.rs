use crate::codegen::bindings;

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
    HttpTrigger(bindings::HttpTrigger),
    Http(bindings::Http),
    TimerTrigger(bindings::TimerTrigger),
    QueueTrigger(bindings::QueueTrigger),
    Queue(bindings::Queue),
    BlobTrigger(bindings::BlobTrigger),
    Blob(bindings::Blob),
    Table(bindings::Table),
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
        }
    }

    pub fn binding_type(&self) -> Option<&str> {
        match self {
            Binding::Context => None,
            Binding::HttpTrigger(_) => Some(bindings::HTTP_TRIGGER_TYPE),
            Binding::Http(_) => Some(bindings::HTTP_TYPE),
            Binding::TimerTrigger(_) => Some(bindings::TIMER_TRIGGER_TYPE),
            Binding::QueueTrigger(_) => Some(bindings::QUEUE_TRIGGER_TYPE),
            Binding::Queue(_) => Some(bindings::QUEUE_TYPE),
            Binding::BlobTrigger(_) => Some(bindings::BLOB_TRIGGER_TYPE),
            Binding::Blob(_) => Some(bindings::BLOB_TYPE),
            Binding::Table(_) => Some(bindings::TABLE_TYPE),
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
            | Binding::BlobTrigger(_) => true,
            Binding::Context
            | Binding::Http(_)
            | Binding::Queue(_)
            | Binding::Blob(_)
            | Binding::Table(_) => false,
        }
    }
}
