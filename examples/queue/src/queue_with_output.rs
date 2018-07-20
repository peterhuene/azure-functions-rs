use azure_functions::bindings::{QueueMessage, QueueTrigger};
use azure_functions::func;

#[func]
#[binding(name = "trigger", queue_name = "echo-in")]
#[binding(name = "$return", queue_name = "echo-out")]
pub fn queue_with_output(trigger: &QueueTrigger) -> QueueMessage {
    let message = trigger.message();

    info!("Message: {}", message);

    message.into()
}
