use azure_functions::{bindings::ServiceBusTrigger, func};

#[func]
#[binding(name = "trigger", queue_name = "example", connection = "connection")]
pub fn log_queue_message(trigger: ServiceBusTrigger) {
    log::info!("{}", trigger.message.as_str().unwrap());
}
