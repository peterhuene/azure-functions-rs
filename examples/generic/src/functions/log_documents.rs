use azure_functions::{bindings::GenericTrigger, func, generic::Value};
use log::info;

#[func]
#[binding(
    type = "cosmosDBTrigger",
    name = "trigger",
    connectionStringSetting = "connection",
    databaseName = "exampledb",
    collectionName = "documents",
    createLeaseCollectionIfNotExists = true
)]
pub fn log_documents(trigger: GenericTrigger) {
    match trigger.data {
        Value::Json(v) => {
            info!("{}", v);
        }
        _ => panic!("expected JSON for Cosmos DB trigger data"),
    }
}
