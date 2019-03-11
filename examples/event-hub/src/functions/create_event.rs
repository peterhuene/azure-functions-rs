use azure_functions::{
    bindings::{HttpRequest, HttpResponse, EventHubMessage},
    func,
};

#[func]
#[binding(name = "output1", connection = "connection", event_hub_name = "example")]
pub fn create_event(_req: HttpRequest) -> (HttpResponse, EventHubMessage) {
    (
        "Created Event Hub message.".into(),
        "Hello from Rust!".into()
    )
}
