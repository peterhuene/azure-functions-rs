use azure_functions::bindings::QueueTrigger;
use azure_functions::func;

#[func]
#[binding(name = "trigger", queue_name = "test")]
pub fn queue(trigger: &QueueTrigger) {
    info!("Message: {}", trigger.message);
}
