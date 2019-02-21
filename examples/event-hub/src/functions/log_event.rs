use azure_functions::{
    bindings::EventHubTrigger,
    func,
};

#[func]
#[binding(name = "trigger", connection = "connection", event_hub_name = "example")]
pub fn log_event(trigger: &EventHubTrigger) {
    log::warn!("Event hub message: {}", trigger.message.as_str().unwrap());
}
