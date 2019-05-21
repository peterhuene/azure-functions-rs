use azure_functions::{bindings::CosmosDbTrigger, func};
use log::info;

#[func]
#[binding(
    name = "trigger",
    connection = "connection",
    database_name = "exampledb",
    collection_name = "documents",
    create_lease_collection = true
)]
pub fn log_documents(trigger: CosmosDbTrigger) {
    for document in trigger.documents {
        info!("{:#?}", document);
    }
}
