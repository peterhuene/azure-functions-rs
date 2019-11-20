use azure_functions::{bindings::CosmosDbTrigger, func};
use log::info;

#[func]
pub fn log_documents(
    #[binding(
        connection = "connection",
        database_name = "exampledb",
        collection_name = "documents",
        create_lease_collection = true
    )]
    trigger: CosmosDbTrigger,
) {
    for document in trigger.documents {
        info!("{:#?}", document);
    }
}
