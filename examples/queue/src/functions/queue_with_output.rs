use azure_functions::{
    bindings::{QueueMessage, QueueTrigger},
    func,
};

#[func]
#[binding(name = "$return", queue_name = "echo-out")]
pub fn queue_with_output(#[binding(queue_name = "echo-in")] trigger: QueueTrigger) -> QueueMessage {
    log::info!("Message: {}", trigger.message);

    trigger.message
}
