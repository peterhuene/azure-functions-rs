use crate::rpc::protocol;
use crate::util::convert_from;
use serde_json::Value;
use std::collections::HashMap;

/// Represents a Cosmos DB trigger binding.
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
/// use log::warn;
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
///         warn!("{}", document);
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
    pub fn new(
        data: protocol::TypedData,
        _metadata: &mut HashMap<String, protocol::TypedData>,
    ) -> Self {
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

        let mut data = protocol::TypedData::new();
        data.set_json(DOCUMENTS.to_string());

        let mut metadata = HashMap::new();
        let trigger = CosmosDbTrigger::new(data, &mut metadata);

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
