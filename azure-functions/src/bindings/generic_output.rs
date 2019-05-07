use crate::{
    generic::Value,
    rpc::{typed_data::Data, TypedData},
};

/// Represents a generic output binding.
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
/// An example of using a `GenericOutput` binding instead of a `CosmosDbDocument` binding:
///
/// ```rust
/// use azure_functions::{
///     bindings::{GenericOutput, HttpRequest, HttpResponse},
///     func,
/// };
/// use serde_json::json;
///
/// #[func]
/// #[binding(name = "req", route = "create/{id}")]
/// #[binding(
///     type = "cosmosDB",
///     name = "output1",
///     connectionStringSetting = "connection",
///     databaseName = "exampledb",
///     collectionName = "documents",
///     createIfNotExists = true
/// )]
/// pub fn create_document(req: HttpRequest) -> (HttpResponse, GenericOutput) {
///     (
///         "Document was created.".into(),
///         json!({
///             "id": req.route_params().get("id").unwrap(),
///             "name": req.query_params().get("name").map_or("stranger", |x| x)
///         })
///         .into(),
///     )
/// }
/// ```
#[derive(Debug, Clone)]
pub struct GenericOutput {
    /// The output binding data.
    pub data: Value,
}

impl From<&str> for GenericOutput {
    fn from(s: &str) -> Self {
        GenericOutput {
            data: Value::String(s.to_owned()),
        }
    }
}

impl From<String> for GenericOutput {
    fn from(s: String) -> Self {
        GenericOutput {
            data: Value::String(s),
        }
    }
}

impl From<serde_json::Value> for GenericOutput {
    fn from(value: serde_json::Value) -> Self {
        GenericOutput {
            data: Value::Json(value),
        }
    }
}

impl From<Vec<u8>> for GenericOutput {
    fn from(bytes: Vec<u8>) -> Self {
        GenericOutput {
            data: Value::Bytes(bytes),
        }
    }
}

impl From<i64> for GenericOutput {
    fn from(integer: i64) -> Self {
        GenericOutput {
            data: Value::Integer(integer),
        }
    }
}

impl From<f64> for GenericOutput {
    fn from(double: f64) -> Self {
        GenericOutput {
            data: Value::Double(double),
        }
    }
}

#[doc(hidden)]
impl Into<TypedData> for GenericOutput {
    fn into(self) -> TypedData {
        match self.data {
            Value::None => TypedData { data: None },
            Value::String(s) => TypedData {
                data: Some(Data::String(s)),
            },
            Value::Json(v) => TypedData {
                data: Some(Data::Json(v.to_string())),
            },
            Value::Bytes(b) => TypedData {
                data: Some(Data::Bytes(b)),
            },
            Value::Integer(i) => TypedData {
                data: Some(Data::Int(i)),
            },
            Value::Double(d) => TypedData {
                data: Some(Data::Double(d)),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::typed_data::Data;
    use serde_json::json;

    #[test]
    fn it_converts_from_str() {
        let s = "hello world";

        let binding: GenericOutput = s.into();

        assert_eq!(binding.data, Value::String(s.to_owned()));
    }

    #[test]
    fn it_converts_from_string() {
        let s = "hello world".to_string();

        let binding: GenericOutput = s.clone().into();

        assert_eq!(binding.data, Value::String(s));
    }

    #[test]
    fn it_converts_from_value() {
        let value = json!({ "foo": "bar" });

        let binding: GenericOutput = value.clone().into();

        assert_eq!(binding.data, Value::Json(value));
    }

    #[test]
    fn it_converts_from_bytes() {
        let value = vec![1, 2, 3];

        let binding: GenericOutput = value.clone().into();

        assert_eq!(binding.data, Value::Bytes(value));
    }

    #[test]
    fn it_converts_from_integer() {
        let value = 12345;

        let binding: GenericOutput = value.into();

        assert_eq!(binding.data, Value::Integer(value));
    }

    #[test]
    fn it_converts_from_double() {
        let value = 12345.6;

        let binding: GenericOutput = value.into();

        assert_eq!(binding.data, Value::Double(value));
    }

    #[test]
    fn it_converts_to_typed_data() {
        let data: TypedData = GenericOutput {
            data: Value::Json(json!({ "foo": "bar" })),
        }
        .into();

        assert_eq!(data.data, Some(Data::Json(r#"{"foo":"bar"}"#.to_owned())));
    }
}
