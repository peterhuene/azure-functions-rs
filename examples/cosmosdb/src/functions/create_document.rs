use azure_functions::{
    bindings::{CosmosDbDocument, HttpRequest, HttpResponse},
    func,
};
use serde_json::json;

#[func]
#[binding(name = "req", route = "create/{id}")]
#[binding(
    name = "output1",
    connection = "connection",
    database_name = "exampledb",
    collection_name = "documents",
    create_collection = true
)]
pub fn create_document(req: HttpRequest) -> (HttpResponse, CosmosDbDocument) {
    (
        "Document was created.".into(),
        json!({
            "id": req.route_params().get("id").unwrap(),
            "name": req.query_params().get("name").map_or("stranger", |x| x)
        })
        .into(),
    )
}
