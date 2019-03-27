use crate::{http::Body, rpc::protocol, util::convert_from, FromVec, IntoVec};
use serde_json::{from_str, Map, Value};
use std::borrow::Cow;
use std::fmt;

/// Represents the input or output binding for a Cosmos DB document.
///
/// The following binding attributes are supported:
///
/// | Name                    | Description                                                                                                                                                                                               |
/// |-------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
/// | `name`                  | The name of the parameter being bound.                                                                                                                                                                    |
/// | `database_name`         | The database containing the document.                                                                                                                                                                     |
/// | `collection_name`       | The name of the collection that contains the document.                                                                                                                                                    |
/// | `id`                    | The identifier of the document to retrieve. This attribute supports binding expressions. Cannot be used with `sql_query`. If neither are specified, the entire collection is retrieved.                   |
/// | `sql_query`             | An Azure Cosmos DB SQL query used for retrieving multiple documents. Cannot be used with `id`. If neither are specified, the entire collection is retrieved.                                              |
/// | `connection`            | The name of the app setting containing your Azure Cosmos DB connection string.                                                                                                                            |
/// | `partition_key`         | Specifies the partition key value for the lookup; may include binding parameters (input only). When `create_collection` is true, defines the partition key path for the created collection (output only). |
/// | `create_collection`     | Specifies if the collection should be created (output only).                                                                                                                                              |
/// | `collection_throughput` | When `create_collection` is true, defines the throughput of the created collection (output only).                                                                                                         |
///
/// # Examples
///
/// Using `CosmosDbDocument` as an input binding with a SQL query:
///
/// ```rust
/// use azure_functions::{
///     bindings::{CosmosDbDocument, HttpRequest, HttpResponse},
///     func,
/// };
///
/// #[func]
/// #[binding(
///     name = "documents",
///     connection = "myconnection",
///     database_name = "mydb",
///     collection_name = "mycollection",
///     sql_query = "select * from mycollection c where startswith(c.name, 'peter')",
/// )]
/// pub fn read_documents(_req: HttpRequest, documents: Vec<CosmosDbDocument>) -> HttpResponse {
///     documents.into()
/// }
/// ```
///
/// Using `CosmosDbDocument` as an input binding for a specific document:
///
/// ```rust
/// use azure_functions::{
///     bindings::{CosmosDbDocument, HttpRequest, HttpResponse},
///     func,
/// };
///
/// #[func]
/// #[binding(name = "_req", route = "read/{id}")]
/// #[binding(
///     name = "document",
///     connection = "myconnection",
///     database_name = "mydb",
///     collection_name = "mycollection",
///     id = "{id}",
/// )]
/// pub fn read_document(_req: HttpRequest, document: CosmosDbDocument) -> HttpResponse {
///     document.into()
/// }
/// ```
///
/// Using `CosmosDbDocument` as an output binding:
///
/// ```rust
/// # use serde_json::json;
/// use azure_functions::{
///     bindings::{CosmosDbDocument, HttpRequest, HttpResponse},
///     func,
/// };
///
/// #[func]
/// #[binding(
///     name = "output1",
///     connection = "myconnection",
///     database_name = "mydb",
///     collection_name = "mycollection"
/// )]
/// pub fn create_document(_req: HttpRequest) -> (HttpResponse, CosmosDbDocument) {
///     (
///         "Document created.".into(),
///         json!({
///             "id": "myid",
///             "name": "Peter",
///             "subject": "example"
///         }).into()
///     )
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
impl IntoVec<CosmosDbDocument> for protocol::TypedData {
    fn into_vec(self) -> Vec<CosmosDbDocument> {
        if self.data.is_none() {
            return vec![];
        }

        match convert_from(&self).expect("expected JSON data for Cosmos DB document") {
            Value::Null => vec![],
            Value::Array(arr) => arr.into_iter().map(CosmosDbDocument::new).collect(),
            Value::Object(obj) => vec![CosmosDbDocument(Value::Object(obj))],
            _ => panic!("expected array or object for Cosmos DB document data"),
        }
    }
}

#[doc(hidden)]
impl FromVec<CosmosDbDocument> for protocol::TypedData {
    fn from_vec(vec: Vec<CosmosDbDocument>) -> Self {
        let mut data = protocol::TypedData::new();
        data.set_json(Value::Array(vec.into_iter().map(|d| d.0).collect()).to_string());
        data
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

impl<'a> Into<Body<'a>> for Vec<CosmosDbDocument> {
    fn into(self) -> Body<'a> {
        Body::Json(Cow::from(
            Value::Array(self.into_iter().map(|d| d.0).collect()).to_string(),
        ))
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
    use serde_json::json;

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
