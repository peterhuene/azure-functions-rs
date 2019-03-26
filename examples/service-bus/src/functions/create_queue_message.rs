use azure_functions::{
    bindings::{HttpRequest, ServiceBusMessage},
    func,
};

#[func]
#[binding(name = "$return", queue_name = "example", connection = "connection")]
pub fn create_queue_message(req: HttpRequest) -> ServiceBusMessage {
    format!(
        "Hello from Rust, {}!\n",
        req.query_params().get("name").map_or("stranger", |x| x)
    )
    .into()
}
