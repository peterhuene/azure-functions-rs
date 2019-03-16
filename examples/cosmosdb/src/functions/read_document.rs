use azure_functions::{
    bindings::{CosmosDbDocument, HttpRequest, HttpResponse},
    func,
};

#[func]
#[binding(name = "req", route = "read/{id}")]
#[binding(
    name = "document",
    connection = "connection",
    database_name = "exampledb",
    collection_name = "documents",
    id = "{id}",
    partition_key = "{id}",
)]
pub fn read_document(req: HttpRequest, document: CosmosDbDocument) -> HttpResponse {
    if document.is_null() {
        format!(
            "Document with id '{}' does not exist.",
            req.route_params().get("id").unwrap()
        )
        .into()
    } else {
        document.into()
    }
}
