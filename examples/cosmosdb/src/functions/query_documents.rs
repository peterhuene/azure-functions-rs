use azure_functions::{
    bindings::{CosmosDbDocument, HttpRequest, HttpResponse},
    func,
};

#[func]
pub fn query_documents(
    #[binding(route = "query/{name}")] _req: HttpRequest,
    #[binding(
        connection = "connection",
        database_name = "exampledb",
        collection_name = "documents",
        sql_query = "select * from documents d where contains(d.name, {name})",
        create_collection = true
    )]
    documents: Vec<CosmosDbDocument>,
) -> HttpResponse {
    documents.into()
}
