use azure_functions::{bindings::GenericTrigger, func, generic::Value};
use log::info;

#[func]
pub fn log_documents(
    #[binding(
        type = "cosmosDBTrigger",
        connectionStringSetting = "connection",
        databaseName = "exampledb",
        collectionName = "documents",
        createLeaseCollectionIfNotExists = true
    )]
    trigger: GenericTrigger,
) {
    match trigger.data {
        Value::Json(v) => {
            info!("{}", v);
        }
        _ => panic!("expected JSON for Cosmos DB trigger data"),
    }
}
