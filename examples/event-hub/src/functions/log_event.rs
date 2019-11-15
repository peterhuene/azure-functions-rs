use azure_functions::{bindings::EventHubTrigger, func};

#[func]
pub fn log_event(
    #[binding(connection = "connection", event_hub_name = "example")] trigger: EventHubTrigger,
) {
    log::info!("Event hub message: {}", trigger.message.to_str().unwrap());
}
