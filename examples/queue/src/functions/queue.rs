use azure_functions::{bindings::QueueTrigger, func};

#[func]
pub fn queue(#[binding(queue_name = "test")] trigger: QueueTrigger) {
    log::info!("Message: {}", trigger.message);
}
