use azure_functions::{
    bindings::{DurableOrchestrationClient, HttpRequest, HttpResponse},
    func,
};

#[func]
pub async fn raise_event(req: HttpRequest, client: DurableOrchestrationClient) -> HttpResponse {
    let id = req
        .query_params()
        .get("id")
        .expect("expected a 'id' parameter");

    let name = req
        .query_params()
        .get("name")
        .expect("expected a 'name' parameter");

    let value = req
        .query_params()
        .get("value")
        .expect("expected a 'value' parameter")
        .clone();

    match client.raise_event(id, name, value).await {
        Ok(_) => format!("Raised event named '{}'.", name).into(),
        Err(e) => format!("Failed to raise event named '{}': {}", name, e).into(),
    }
}
