use azure_functions::{bindings::ServiceBusTrigger, func};

#[func]
#[binding(
    name = "trigger",
    topic_name = "mytopic",
    subscription_name = "mysubscription",
    connection = "connection"
)]
pub fn log_topic_message(trigger: ServiceBusTrigger) {
    log::info!("{}", trigger.message.as_str().unwrap());
}
