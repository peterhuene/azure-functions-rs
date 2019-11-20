use azure_functions::{
    bindings::{CosmosDbDocument, HttpRequest, HttpResponse},
    func,
};

#[func]
pub fn read_document(
    #[binding(route = "read/{id}")] req: HttpRequest,
    #[binding(
        connection = "connection",
        database_name = "exampledb",
        collection_name = "documents",
        id = "{id}",
        partition_key = "{id}"
    )]
    document: CosmosDbDocument,
) -> HttpResponse {
    if document.is_null() {
        format!(
            "Document with id '{}' does not exist.",
            req.route_params.get("id").unwrap()
        )
        .into()
    } else {
        document.into()
    }
}
