use azure_functions::{
    bindings::{DurableOrchestrationClient, HttpRequest, HttpResponse},
    func,
};
use serde_json::Value;

#[func]
pub async fn start(req: HttpRequest, client: DurableOrchestrationClient) -> HttpResponse {
    match client
        .start_new(
            req.query_params()
                .get("function")
                .expect("expected a function parameter"),
            None,
            Value::Null,
        )
        .await
    {
        Ok(data) => format!("Orchestration started: {}", data.status_query_get_uri).into(),
        Err(e) => format!("Failed to start orchestration: {}", e).into(),
    }
}
