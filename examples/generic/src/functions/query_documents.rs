use azure_functions::{
    bindings::{GenericInput, HttpRequest, HttpResponse},
    func,
};

#[func]
pub fn query_documents(
    #[binding(route = "query/{name}")] _req: HttpRequest,
    #[binding(
        type = "cosmosDB",
        connectionStringSetting = "connection",
        databaseName = "exampledb",
        collectionName = "documents",
        sqlQuery = "select * from documents d where contains(d.name, {name})",
        createIfNotExists = true
    )]
    documents: GenericInput,
) -> HttpResponse {
    documents.into()
}
