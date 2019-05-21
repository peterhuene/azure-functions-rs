use crate::{rpc::TypedData, util::convert_from};
use serde_json::Value;
use std::collections::HashMap;

/// Represents a Cosmos DB trigger binding.
///
/// The following binding attributes are supported:
///
/// | Name                          | Description                                                                                                                                                                                                                                                                                                                      |
/// |-------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
/// | `name`                        | The name of the parameter being bound.                                                                                                                                                                                                                                                                                           |
/// | `connection`                  | The name of an app setting that contains the connection string used to connect to the Azure Cosmos DB account being monitored.                                                                                                                                                                                                   |
/// | `database_name`               | The name of the Azure Cosmos DB database with the collection being monitored.                                                                                                                                                                                                                                                    |
/// | `collection_name`             | The name of the collection being monitored.                                                                                                                                                                                                                                                                                      |
/// | `lease_connection`            | The name of an app setting that contains the connection string to the service which holds the lease collection. When not set, the `connection` value is used.                                                                                                                                                                    |
/// | `lease_database_name`         | The name of the database that holds the collection used to store leases. When not set, the value of the `database_name` setting is used.                                                                                                                                                                                         |
/// | `create_lease_collection`     | When set to true, the leases collection is automatically created when it doesn't already exist. The default value is false.                                                                                                                                                                                                      |
/// | `lease_collection_throughput` | Defines the amount of Request Units to assign when the leases collection is created (optional). This setting is only used when `create_lease_collection` is set to true.                                                                                                                                                         |
/// | `lease_collection_prefix`     | When set, it adds a prefix to the leases created in the Lease collection for this Function, effectively allowing two separate Azure Functions to share the same Lease collection by using different prefixes.                                                                                                                    |
/// | `feed_poll_delay`             | When set, it defines, in milliseconds, the delay in between polling a partition for new changes on the feed, after all current changes are drained. Default is 5000 (5 seconds).                                                                                                                                                 |
/// | `lease_acquire_interval`      | When set, it defines, in milliseconds, the interval to kick off a task to compute if partitions are distributed evenly among known host instances. Default is 13000 (13 seconds).                                                                                                                                                |
/// | `lease_expiration_interval`   | When set, it defines, in milliseconds, the interval for which the lease is taken on a lease representing a partition. If the lease is not renewed within this interval, it will cause it to expire and ownership of the partition will move to another instance. Default is 60000 (60 seconds).                                  |
/// | `lease_renew_interval`        | When set, it defines, in milliseconds, the renew interval for all leases for partitions currently held by an instance. Default is 17000 (17 seconds).                                                                                                                                                                            |
/// | `checkpoint_frequency`        | When set, it defines, in milliseconds, the interval between lease checkpoints. Default is always after each Function call.                                                                                                                                                                                                       |
/// | `max_items_per_invocation`    | When set, it customizes the maximum amount of items received per Function call.                                                                                                                                                                                                                                                  |
/// | `start_from_beginning`        | When set, it tells the Trigger to start reading changes from the beginning of the history of the collection instead of the current time. This only works the first time the Trigger starts, as in subsequent runs, the checkpoints are already stored. Setting this to true when there are leases already created has no effect. |
///
/// # Examples
///
/// An example of logging all Cosmos DB documents that triggered the function:
///
/// ```rust
/// use azure_functions::{
///     bindings::CosmosDbTrigger,
///     func,
/// };
/// use log::info;
///
/// #[func]
/// #[binding(
///     name = "trigger",
///     connection = "myconnection",
///     database_name = "mydb",
///     collection_name = "mycollection"
/// )]
/// pub fn log_documents(trigger: CosmosDbTrigger) {
///     for document in trigger.documents {
///         info!("{}", document);
///     }
/// }
/// ```
#[derive(Debug)]
pub struct CosmosDbTrigger {
    /// The Cosmos DB documents that triggered the function.
    pub documents: Vec<Value>,
}

impl CosmosDbTrigger {
    #[doc(hidden)]
    pub fn new(data: TypedData, _metadata: HashMap<String, TypedData>) -> Self {
        let value = convert_from(&data).expect("expected JSON document data");
        match value {
            Value::Array(array) => CosmosDbTrigger { documents: array },
            _ => panic!("expected a JSON array for Cosmos DB trigger data"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::typed_data::Data;

    #[test]
    fn it_constructs() {
        const DOCUMENTS: &'static str = r#"[
            {
                "id": "id1",
                "_etag": "etag1",
                "_rid": "rid1",
                "_self": "self1",
                "_ts": 1,
                "data": "value1"
            },
            {
                "id": "id2",
                "_etag": "etag2",
                "_rid": "rid2",
                "_self": "self2",
                "_ts": 2,
                "data": "value2"
            },
            {
                "id": "id3",
                "_etag": "etag3",
                "_rid": "rid3",
                "_self": "self3",
                "_ts": 3,
                "data": "value3"
            }
        ]"#;

        let data = TypedData {
            data: Some(Data::Json(DOCUMENTS.to_string())),
        };

        let trigger = CosmosDbTrigger::new(data, HashMap::new());

        let documents = trigger.documents;
        assert_eq!(documents.len(), 3);

        assert_eq!(documents[0]["id"].as_str().unwrap(), "id1");
        assert_eq!(documents[0]["_etag"].as_str().unwrap(), "etag1");
        assert_eq!(documents[0]["_rid"].as_str().unwrap(), "rid1");
        assert_eq!(documents[0]["_self"].as_str().unwrap(), "self1");
        assert_eq!(documents[0]["_ts"].as_i64().unwrap(), 1);
        assert_eq!(documents[0]["data"].as_str().unwrap(), "value1");

        assert_eq!(documents[1]["id"].as_str().unwrap(), "id2");
        assert_eq!(documents[1]["_etag"].as_str().unwrap(), "etag2");
        assert_eq!(documents[1]["_rid"].as_str().unwrap(), "rid2");
        assert_eq!(documents[1]["_self"].as_str().unwrap(), "self2");
        assert_eq!(documents[1]["_ts"].as_i64().unwrap(), 2);
        assert_eq!(documents[1]["data"].as_str().unwrap(), "value2");

        assert_eq!(documents[2]["id"].as_str().unwrap(), "id3");
        assert_eq!(documents[2]["_etag"].as_str().unwrap(), "etag3");
        assert_eq!(documents[2]["_rid"].as_str().unwrap(), "rid3");
        assert_eq!(documents[2]["_self"].as_str().unwrap(), "self3");
        assert_eq!(documents[2]["_ts"].as_i64().unwrap(), 3);
        assert_eq!(documents[2]["data"].as_str().unwrap(), "value3");
    }
}
