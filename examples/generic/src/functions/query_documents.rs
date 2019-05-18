use azure_functions::{
    bindings::{GenericInput, HttpRequest, HttpResponse},
    func,
};

#[func]
#[binding(name = "_req", route = "query/{name}")]
#[binding(
    type = "cosmosDB",
    name = "documents",
    connectionStringSetting = "connection",
    databaseName = "exampledb",
    collectionName = "documents",
    sqlQuery = "select * from documents d where contains(d.name, {name})",
    createIfNotExists = true
)]
pub fn query_documents(_req: HttpRequest, documents: GenericInput) -> HttpResponse {
    documents.into()
}
