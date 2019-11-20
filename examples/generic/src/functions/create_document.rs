use azure_functions::{
    bindings::{GenericOutput, HttpRequest, HttpResponse},
    func,
};
use serde_json::json;

#[func]
#[binding(
    type = "cosmosDB",
    name = "output1",
    connectionStringSetting = "connection",
    databaseName = "exampledb",
    collectionName = "documents",
    createIfNotExists = true
)]
pub fn create_document(
    #[binding(route = "create/{id}")] mut req: HttpRequest,
) -> (HttpResponse, GenericOutput) {
    (
        "Document was created.".into(),
        json!({
            "id": req.route_params.remove("id").unwrap(),
            "name": req.query_params.remove("name").expect("expected a 'name' query parameter"),
        })
        .into(),
    )
}
