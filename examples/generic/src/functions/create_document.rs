use azure_functions::{
    bindings::{GenericOutput, HttpRequest, HttpResponse},
    func,
};
use serde_json::json;

#[func]
#[binding(name = "req", route = "create/{id}")]
#[binding(
    type = "cosmosDB",
    name = "output1",
    connectionStringSetting = "connection",
    databaseName = "exampledb",
    collectionName = "documents",
    createIfNotExists = true
)]
pub fn create_document(req: HttpRequest) -> (HttpResponse, GenericOutput) {
    (
        "Document was created.".into(),
        json!({
            "id": req.route_params().get("id").unwrap(),
            "name": req.query_params().get("name").map_or("stranger", |x| x)
        })
        .into(),
    )
}
