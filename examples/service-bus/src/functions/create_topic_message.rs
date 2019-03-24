use azure_functions::{
    bindings::{HttpRequest, ServiceBusMessage},
    func,
};

#[func]
#[binding(
    name = "$return",
    topic_name = "mytopic",
    subscription_name = "mysubscription",
    connection = "connection"
)]
pub fn create_topic_message(req: HttpRequest) -> ServiceBusMessage {
    format!(
        "Hello from Rust, {}!\n",
        req.query_params().get("name").map_or("stranger", |x| x)
    )
    .into()
}
