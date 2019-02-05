use azure_functions::{bindings::QueueTrigger, func};

#[func]
#[binding(name = "trigger", queue_name = "test")]
pub fn queue(trigger: &QueueTrigger) {
    log::info!("Message: {}", trigger.message);
}
