use azure_functions::{bindings::EventGridEvent, func};

#[func]
pub fn log_event(event: EventGridEvent) {
    log::info!("Event Data: {}", event.data);
}
