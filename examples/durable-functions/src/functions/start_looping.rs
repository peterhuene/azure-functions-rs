use azure_functions::{
    bindings::{DurableOrchestrationClient, HttpRequest, HttpResponse},
    func,
};

#[func]
pub async fn start_looping(_req: HttpRequest, client: DurableOrchestrationClient) -> HttpResponse {
    match client.start_new("looping", None, 0).await {
        Ok(_) => "Orchestration started.".into(),
        Err(e) => format!("Failed to start orchestration: {}", e).into(),
    }
}
