use azure_functions::{
    bindings::{CosmosDbDocuments, HttpRequest, HttpResponse},
    func,
};

#[func]
#[binding(name = "req", route = "read/{id}")]
#[binding(
    name = "documents",
    connection = "connection",
    database_name = "exampledb",
    collection_name = "documents",
    id = "{id}",
    partition_key = "{id}",
)]
pub fn read_document(req: HttpRequest, documents: CosmosDbDocuments) -> HttpResponse {
    if documents.is_empty() {
        format!(
            "Document with id '{}' does not exist.",
            req.route_params().get("id").unwrap()
        )
        .into()
    } else {
        documents.into_iter().nth(0).unwrap().into()
    }
}
