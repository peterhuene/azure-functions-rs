use azure_functions::{bindings::ServiceBusTrigger, func};

#[func]
pub fn log_queue_message(
    #[binding(queue_name = "example", connection = "connection")] trigger: ServiceBusTrigger,
) {
    log::info!("{}", trigger.message.to_str().unwrap());
}
