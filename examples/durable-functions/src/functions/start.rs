use azure_functions::{
    bindings::{DurableOrchestrationClient, HttpRequest, HttpResponse},
    func,
};

#[func]
pub async fn start(_req: HttpRequest, _client: DurableOrchestrationClient) -> HttpResponse {
    // match client.start_new("HelloWorld").await {
    //     Ok(_) => "Orchestration started.".into(),
    //     Err(e) => format!("Failed to start orchestration: {}", e).into()
    // }
    unimplemented!()
}
