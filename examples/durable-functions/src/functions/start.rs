use azure_functions::{
    bindings::{DurableOrchestrationClient, HttpRequest, HttpResponse},
    func,
};
use serde_json::Value;

#[func]
pub async fn start(_req: HttpRequest, client: DurableOrchestrationClient) -> HttpResponse {
    match client.start_new("hello_world", None, Value::Null).await {
        Ok(_) => "Orchestration started.".into(),
        Err(e) => format!("Failed to start orchestration: {}", e).into(),
    }
}
