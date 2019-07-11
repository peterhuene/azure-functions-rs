use azure_functions::{
    bindings::{CosmosDbDocument, HttpRequest, HttpResponse},
    func,
};

#[func]
#[binding(name = "_req", route = "query/{name}")]
#[binding(
    name = "documents",
    connection = "connection",
    database_name = "exampledb",
    collection_name = "documents",
    sql_query = "select * from documents d where contains(d.name, {name})",
    create_collection = true
)]
pub fn query_documents(_req: HttpRequest, documents: Vec<CosmosDbDocument>) -> HttpResponse {
    documents.into()
}
