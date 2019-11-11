use azure_functions::{
    bindings::{DurableOrchestrationClient, HttpRequest, HttpResponse},
    func,
};

#[func]
pub async fn start_looping(_req: HttpRequest, client: DurableOrchestrationClient) -> HttpResponse {
    match client.start_new("looping", None, 0).await {
        Ok(data) => format!("Orchestration started: {}", data.status_query_get_uri).into(),
        Err(e) => format!("Failed to start orchestration: {}", e).into(),
    }
}
