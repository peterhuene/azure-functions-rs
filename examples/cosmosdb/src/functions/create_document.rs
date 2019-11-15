use azure_functions::{
    bindings::{CosmosDbDocument, HttpRequest, HttpResponse},
    func,
};
use serde_json::json;

#[func]
#[binding(
    name = "output1",
    connection = "connection",
    database_name = "exampledb",
    collection_name = "documents",
    create_collection = true
)]
pub fn create_document(
    #[binding(route = "create/{id}")] mut req: HttpRequest,
) -> (HttpResponse, CosmosDbDocument) {
    (
        "Document was created.".into(),
        json!({
            "id": req.route_params.remove("id").unwrap(),
            "name": req.query_params.remove("name").expect("expected a 'name' query parameter"),
        })
        .into(),
    )
}
