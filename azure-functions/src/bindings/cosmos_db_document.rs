use crate::http::Body;
use crate::rpc::protocol;
use crate::util::convert_from;
use serde_json::{from_str, Map, Value};
use std::fmt;

/// Represents the input or output binding for a Cosmos DB document.
///
/// # Examples
///
/// Using `CosmosDbDocument` as an input binding:
///
/// ```rust
/// # extern crate azure_functions;
/// # #[macro_use] extern crate log;
/// use azure_functions::{
///     bindings::{CosmosDbDocument, HttpRequest, HttpResponse},
///     func,
/// };
///
/// #[func]
/// #[binding(name = "_req", route = "read/{id}")]
/// #[binding(
///     name = "doc",
///     connection = "myconnection",
///     database_name = "mydb",
///     collection_name = "mycollection",
///     id = "{id}"
/// )]
/// pub fn read_documents(_req: HttpRequest, doc: CosmosDbDocument) -> HttpResponse {
///     doc.into()
/// }
/// ```
///
/// Using `CosmosDbDocument` as an output binding:
///
/// ```rust
/// # extern crate azure_functions;
/// # use serde_json::json;
/// use azure_functions::{
///     bindings::{CosmosDbDocument, HttpRequest},
///     func,
/// };
///
/// #[func]
/// #[binding(
///     name = "$return",
///     connection = "myconnection",
///     database_name = "mydb",
///     collection_name = "mycollection"
/// )]
/// pub fn create_document(_req: HttpRequest) -> CosmosDbDocument {
///     json!({
///         "id": "myid",
///         "name": "Peter",
///         "subject": "example"
///     }).into()
/// }
/// ```
#[derive(Debug, Clone)]
pub struct CosmosDbDocument(Value);

impl CosmosDbDocument {
    /// Creates a new `CosmosDbDocument` from a JSON object value.
    ///
    /// The value must be a JSON object.
    pub fn new(value: Value) -> CosmosDbDocument {
        if !value.is_object() {
            panic!("expected a single object for a Cosmos DB document");
        }
        CosmosDbDocument(value)
    }

    /// Gets whether or not the Cosmos DB document is null.
    ///
    /// A Cosmos DB document can be null as a result of a query that returned no documents.
    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }

    /// Gets the JSON object for the Cosmos DB document
    ///
    /// Returns None if the document is null.
    pub fn as_object(&self) -> Option<&Map<String, Value>> {
        self.0.as_object()
    }

    /// Gets a mutable JSON object for the Cosmos DB document
    ///
    /// Returns None if the document is null.
    pub fn as_object_mut(&mut self) -> Option<&mut Map<String, Value>> {
        self.0.as_object_mut()
    }
}

impl fmt::Display for CosmosDbDocument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a> From<&'a str> for CosmosDbDocument {
    fn from(json: &'a str) -> Self {
        CosmosDbDocument::new(from_str(json).unwrap())
    }
}

impl From<String> for CosmosDbDocument {
    fn from(json: String) -> Self {
        CosmosDbDocument::new(from_str(&json).unwrap())
    }
}

impl From<Value> for CosmosDbDocument {
    fn from(value: Value) -> Self {
        CosmosDbDocument::new(value)
    }
}

#[doc(hidden)]
impl From<protocol::TypedData> for CosmosDbDocument {
    fn from(data: protocol::TypedData) -> Self {
        if data.data.is_none() {
            return CosmosDbDocument(Value::Null);
        }

        let value: Value = convert_from(&data).expect("expected JSON data for Cosmos DB document");

        match value {
            Value::Null => CosmosDbDocument(Value::Null),
            Value::Array(mut arr) => {
                if arr.is_empty() {
                    CosmosDbDocument(Value::Null)
                } else {
                    CosmosDbDocument::new(arr.swap_remove(0))
                }
            }
            Value::Object(obj) => CosmosDbDocument(Value::Object(obj)),
            _ => panic!("expected an array or object for Cosmos DB document data"),
        }
    }
}

impl Into<String> for CosmosDbDocument {
    fn into(self) -> String {
        self.0.to_string()
    }
}

impl Into<Value> for CosmosDbDocument {
    fn into(self) -> Value {
        self.0
    }
}

impl<'a> Into<Body<'a>> for CosmosDbDocument {
    fn into(self) -> Body<'a> {
        self.0.into()
    }
}

#[doc(hidden)]
impl Into<protocol::TypedData> for CosmosDbDocument {
    fn into(self) -> protocol::TypedData {
        let mut data = protocol::TypedData::new();
        data.set_json(self.0.to_string());
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_constructs_from_an_object_value() {
        let document = CosmosDbDocument::new(json!({ "id": "foo", "key": "value"}));
        let data = document.as_object().unwrap();
        assert_eq!(data["id"].as_str().unwrap(), "foo");
        assert_eq!(data["key"].as_str().unwrap(), "value");
    }

    #[test]
    #[should_panic(expected = "expected a single object for a Cosmos DB document")]
    fn it_panics_if_constructed_without_an_object_or_array() {
        CosmosDbDocument::new(json!(5));
    }

    #[test]
    fn it_displays_as_json() {
        let document = CosmosDbDocument::new(json!({ "foo": "bar"}));
        assert_eq!(format!("{}", document), r#"{"foo":"bar"}"#);
    }

    #[test]
    fn it_converts_from_str() {
        let document: CosmosDbDocument = r#"{ "foo": "bar" }"#.into();
        let data = document.as_object().unwrap();
        assert_eq!(data["foo"].as_str().unwrap(), "bar");
    }

    #[test]
    fn it_converts_from_string() {
        let document: CosmosDbDocument = r#"{ "foo": "bar" }"#.to_string().into();
        let data = document.as_object().unwrap();
        assert_eq!(data["foo"].as_str().unwrap(), "bar");
    }

    #[test]
    fn it_converts_from_value() {
        let document: CosmosDbDocument = json!({ "foo": "bar" }).into();
        let data = document.as_object().unwrap();
        assert_eq!(data["foo"].as_str().unwrap(), "bar");
    }

    #[test]
    fn it_converts_to_string() {
        let document: CosmosDbDocument = json!({ "foo": "bar" }).into();
        let string: String = document.into();
        assert_eq!(string, r#"{"foo":"bar"}"#);
    }

    #[test]
    fn it_converts_to_value() {
        let document: CosmosDbDocument = json!({ "foo": "bar" }).into();
        let data = document.as_object().unwrap();
        assert_eq!(data["foo"].as_str().unwrap(), "bar");

        let value: Value = document.into();
        assert!(value.is_object());
        assert_eq!(value.as_object().unwrap()["foo"].as_str().unwrap(), "bar");
    }

    #[test]
    fn it_converts_to_body() {
        let document: CosmosDbDocument = r#"{ "foo": "bar" }"#.into();
        let body: Body = document.into();
        assert_eq!(body.as_str().unwrap(), r#"{"foo":"bar"}"#);

        let document: CosmosDbDocument = json!({"hello": "world"}).into();
        let body: Body = document.into();
        assert_eq!(body.as_str().unwrap(), r#"{"hello":"world"}"#);
    }

    #[test]
    fn it_converts_from_typed_data() {
        let mut data = protocol::TypedData::new();
        data.set_json(r#"{ "foo": "bar" }"#.to_string());

        let document: CosmosDbDocument = data.into();
        let data = document.as_object().unwrap();
        assert_eq!(data["foo"].as_str().unwrap(), "bar");
    }

    #[test]
    fn it_converts_to_typed_data() {
        let document: CosmosDbDocument = json!({ "foo": "bar" }).into();
        let data = document.as_object().unwrap();
        assert_eq!(data["foo"].as_str().unwrap(), "bar");

        let data: protocol::TypedData = document.into();
        assert!(data.has_json());
        assert_eq!(data.get_json(), r#"{"foo":"bar"}"#);
    }
}
