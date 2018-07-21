use codegen::bindings::{Http, HttpTrigger, TimerTrigger};

#[derive(Serialize, Debug, Clone)]
#[serde(untagged, rename_all = "camelCase")]
pub enum Binding {
    Context,
    HttpTrigger(HttpTrigger),
    Http(Http),
    TimerTrigger(TimerTrigger),
}

impl Binding {
    pub fn name(&self) -> Option<&str> {
        match self {
            Binding::Context => None,
            Binding::HttpTrigger(b) => Some(&b.name),
            Binding::Http(b) => Some(&b.name),
            Binding::TimerTrigger(b) => Some(&b.name),
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
            Binding::HttpTrigger(_) | Binding::TimerTrigger(_) => true,
            Binding::Context | Binding::Http(_) => false,
        }
    }
}
