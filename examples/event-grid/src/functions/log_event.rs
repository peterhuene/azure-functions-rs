use azure_functions::{bindings::EventGridEvent, func};

#[func]
pub fn log_event(event: EventGridEvent) {
    log::warn!("Event Data: {}", event.data);
}
