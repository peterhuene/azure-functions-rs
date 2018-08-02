use azure_functions::bindings::{QueueMessage, QueueTrigger};
use azure_functions::func;

#[func]
#[binding(name = "trigger", queue_name = "echo-in")]
#[binding(name = "$return", queue_name = "echo-out")]
pub fn queue_with_output(trigger: &QueueTrigger) -> QueueMessage {
    info!("Message: {}", trigger.message);

    trigger.message.clone()
}
