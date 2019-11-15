use azure_functions::{bindings::ServiceBusTrigger, func};

#[func]
pub fn log_topic_message(
    #[binding(
        topic_name = "mytopic",
        subscription_name = "mysubscription",
        connection = "connection"
    )]
    trigger: ServiceBusTrigger,
) {
    log::info!("{}", trigger.message.to_str().unwrap());
}
