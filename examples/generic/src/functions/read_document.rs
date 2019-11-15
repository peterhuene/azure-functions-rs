use azure_functions::{
    bindings::{GenericInput, HttpRequest, HttpResponse},
    func,
    generic::Value,
};
use serde_json::from_str;

#[func]
pub fn read_document(
    #[binding(route = "read/{id}")] req: HttpRequest,
    #[binding(
        type = "cosmosDB",
        connectionStringSetting = "connection",
        databaseName = "exampledb",
        collectionName = "documents",
        id = "{id}",
        partitionKey = "{id}"
    )]
    document: GenericInput,
) -> HttpResponse {
    match document.data {
        Value::String(s) => {
            let v: serde_json::Value = from_str(&s).expect("expected JSON data");
            if v.is_null() {
                format!(
                    "Document with id '{}' does not exist.",
                    req.route_params.get("id").unwrap()
                )
                .into()
            } else {
                v.into()
            }
        }
        _ => panic!("expected string for CosmosDB document data"),
    }
}
