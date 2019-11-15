use crate::{generic::Value, rpc::TypedData};
use std::collections::HashMap;

/// Represents a generic trigger binding.
///
/// The following binding attributes are supported:
///
/// | Name                    | Description                                                                                                                |
/// |-------------------------|----------------------------------------------------------------------------------------------------------------------------|
/// | `type`                  | The binding type.                                                                                                          |
/// | `name`                  | The name of the parameter being bound.                                                                                     |
/// | `*`                     | The additional binding attributes specific to the binding type. Supported value types are strings, booleans, and integers. |
///
/// # Examples
///
/// An example of using a `GenericTrigger` binding instead of a `CosmosDbTrigger` binding:
///
/// ```rust
/// use azure_functions::{bindings::GenericTrigger, func, generic::Value};
/// use log::info;
///
/// #[func]
/// #[binding(
///     type = "cosmosDBTrigger",
///     name = "trigger",
///     connectionStringSetting = "connection",
///     databaseName = "exampledb",
///     collectionName = "documents",
///     createLeaseCollectionIfNotExists = true
/// )]
/// pub fn log_documents(trigger: GenericTrigger) {
///     match trigger.data {
///         Value::Json(v) => {
///             info!("{}", v);
///         }
///         _ => panic!("expected JSON for Cosmos DB trigger data"),
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct GenericTrigger {
    /// The trigger binding data.
    pub data: Value,
    /// The trigger metadata.
    pub metadata: HashMap<String, Value>,
}

impl GenericTrigger {
    #[doc(hidden)]
    pub fn new(data: TypedData, metadata: HashMap<String, TypedData>) -> Self {
        let mut md = HashMap::with_capacity(metadata.len());
        for (k, v) in metadata.into_iter() {
            md.insert(k, v.into());
        }
        Self {
            data: data.into(),
            metadata: md,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::typed_data::Data;
    use serde_json::json;

    #[test]
    fn it_constructs() {
        let data = TypedData {
            data: Some(Data::Json(r#"{ "foo": "bar" }"#.to_string())),
        };

        let mut metadata = HashMap::new();
        metadata.insert(
            "foo".to_string(),
            TypedData {
                data: Some(Data::String("bar".to_string())),
            },
        );

        let binding = GenericTrigger::new(data, metadata);

        assert_eq!(binding.data, Value::Json(json!({ "foo": "bar" })));
        assert_eq!(binding.metadata.len(), 1);
        assert_eq!(
            binding.metadata.get("foo"),
            Some(&Value::String("bar".to_string()))
        );
    }
}
