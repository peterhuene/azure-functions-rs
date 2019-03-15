use crate::http::Body;
use crate::rpc::protocol;
use crate::util::convert_from;
use serde_json::{from_str, Value};
use std::fmt;
use std::ops::{Index, IndexMut};
use std::slice::{Iter, SliceIndex};
use std::vec::IntoIter;

/// Represents the input or output binding for a collection of Cosmos DB documents.
///
/// # Examples
///
/// Using `CosmosDbDocuments` as an input binding:
///
/// ```rust
/// # extern crate azure_functions;
/// # #[macro_use] extern crate log;
/// use azure_functions::{
///     bindings::{CosmosDbDocuments, HttpRequest, HttpResponse},
///     func,
/// };
///
/// #[func]
/// #[binding(name = "_req", route = "read/{id}")]
/// #[binding(
///     name = "docs",
///     connection = "myconnection",
///     database_name = "mydb",
///     collection_name = "mycollection",
///     sql_query = "select * from mycollection c where startswith(c.name, 'peter')"
/// )]
/// pub fn read_documents(_req: HttpRequest, docs: CosmosDbDocuments) -> HttpResponse {
///     docs.into()
/// }
/// ```
///
/// Using `CosmosDbDocuments` as an output binding:
///
/// ```rust
/// # extern crate azure_functions;
/// # use serde_json::json;
/// use azure_functions::{
///     bindings::{CosmosDbDocuments, HttpRequest},
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
/// pub fn create_document(_req: HttpRequest) -> CosmosDbDocuments {
///     json!({
///         "id": "myid",
///         "name": "Peter",
///         "subject": "example"
///     }).into()
/// }
/// ```
#[derive(Debug, Clone)]
pub struct CosmosDbDocuments(Value);

impl CosmosDbDocuments {
    fn new(value: Value) -> CosmosDbDocuments {
        match value {
            Value::Object(value) => CosmosDbDocuments(Value::Array(vec![Value::Object(value)])),
            Value::Array(arr) => {
                if !arr.iter().all(Value::is_object) {
                    panic!("expected either a single object or an array of objects");
                }
                CosmosDbDocuments(Value::Array(arr))
            }
            _ => panic!("expected either a single object or an array of objects"),
        }
    }

    /// Gets the length of the Cosmos DB documents stored in the collection.
    pub fn len(&self) -> usize {
        self.0.as_array().unwrap().len()
    }

    /// Gets whether or not the collection of Cosmos DB documents is empty.
    pub fn is_empty(&self) -> bool {
        self.0.as_array().unwrap().is_empty()
    }

    /// Gets an iterator over the Cosmos DB documents in the collection.
    pub fn iter(&self) -> Iter<Value> {
        self.0.as_array().unwrap().iter()
    }
}

impl fmt::Display for CosmosDbDocuments {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<I> Index<I> for CosmosDbDocuments
where
    I: SliceIndex<[Value]>,
{
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.0.as_array().unwrap()[index]
    }
}

impl<I> IndexMut<I> for CosmosDbDocuments
where
    I: SliceIndex<[Value]>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.0.as_array_mut().unwrap()[index]
    }
}

impl<'a> IntoIterator for CosmosDbDocuments {
    type Item = Value;
    type IntoIter = IntoIter<Value>;

    fn into_iter(self) -> Self::IntoIter {
        match self.0 {
            Value::Array(arr) => arr.into_iter(),
            _ => panic!("unexpected value"),
        }
    }
}

impl<'a> From<&'a str> for CosmosDbDocuments {
    fn from(json: &'a str) -> Self {
        CosmosDbDocuments::new(from_str(json).unwrap())
    }
}

impl From<String> for CosmosDbDocuments {
    fn from(json: String) -> Self {
        CosmosDbDocuments::new(from_str(&json).unwrap())
    }
}

impl From<Value> for CosmosDbDocuments {
    fn from(value: Value) -> Self {
        CosmosDbDocuments::new(value)
    }
}

impl From<Vec<Value>> for CosmosDbDocuments {
    fn from(values: Vec<Value>) -> Self {
        CosmosDbDocuments::new(Value::Array(values))
    }
}

#[doc(hidden)]
impl From<protocol::TypedData> for CosmosDbDocuments {
    fn from(data: protocol::TypedData) -> Self {
        if data.data.is_none() {
            return CosmosDbDocuments(Value::Array(vec![]));
        }
        CosmosDbDocuments::new(
            convert_from(&data).expect("expected JSON data for Cosmos DB document"),
        )
    }
}

impl Into<String> for CosmosDbDocuments {
    fn into(self) -> String {
        self.0.to_string()
    }
}

impl Into<Value> for CosmosDbDocuments {
    fn into(self) -> Value {
        self.0
    }
}

impl Into<Vec<Value>> for CosmosDbDocuments {
    fn into(self) -> Vec<Value> {
        match self.0 {
            Value::Array(arr) => arr,
            _ => panic!("unexpected value"),
        }
    }
}

impl<'a> Into<Body<'a>> for CosmosDbDocuments {
    fn into(self) -> Body<'a> {
        self.0.into()
    }
}

#[doc(hidden)]
impl Into<protocol::TypedData> for CosmosDbDocuments {
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
        let documents = CosmosDbDocuments::new(json!({ "id": "foo", "key": "value"}));
        assert_eq!(documents.len(), 1);
        assert_eq!(documents[0]["id"].as_str().unwrap(), "foo");
        assert_eq!(documents[0]["key"].as_str().unwrap(), "value");
    }

    #[test]
    fn it_constructs_from_an_array_value() {
        let documents = CosmosDbDocuments::new(
            json!([{ "id": "foo", "key": "value"}, { "id": "bar", "key": "value2"}]),
        );
        assert_eq!(documents.len(), 2);
        assert_eq!(documents[0]["id"].as_str().unwrap(), "foo");
        assert_eq!(documents[0]["key"].as_str().unwrap(), "value");
        assert_eq!(documents[1]["id"].as_str().unwrap(), "bar");
        assert_eq!(documents[1]["key"].as_str().unwrap(), "value2");
    }

    #[test]
    #[should_panic(expected = "expected either a single object or an array of objects")]
    fn it_panics_if_constructed_without_an_object_or_array() {
        CosmosDbDocuments::new(json!(5));
    }

    #[test]
    #[should_panic(expected = "expected either a single object or an array of objects")]
    fn it_panics_if_constructed_with_array_of_nonobjects() {
        CosmosDbDocuments::new(json!([5]));
    }

    #[test]
    fn it_displays_as_json() {
        let documents = CosmosDbDocuments::new(json!({ "foo": "bar"}));
        assert_eq!(format!("{}", documents), r#"[{"foo":"bar"}]"#);
    }

    #[test]
    fn it_converts_from_str() {
        let documents: CosmosDbDocuments = r#"{ "foo": "bar" }"#.into();
        assert_eq!(documents.len(), 1);
        assert_eq!(documents[0]["foo"].as_str().unwrap(), "bar");
    }

    #[test]
    fn it_converts_from_string() {
        let documents: CosmosDbDocuments = r#"{ "foo": "bar" }"#.to_string().into();
        assert_eq!(documents.len(), 1);
        assert_eq!(documents[0]["foo"].as_str().unwrap(), "bar");
    }

    #[test]
    fn it_converts_from_value() {
        let documents: CosmosDbDocuments = json!({ "foo": "bar" }).into();
        assert_eq!(documents.len(), 1);
        assert_eq!(documents[0]["foo"].as_str().unwrap(), "bar");
    }

    #[test]
    fn it_converts_to_string() {
        let documents: CosmosDbDocuments = json!({ "foo": "bar" }).into();
        let string: String = documents.into();
        assert_eq!(string, r#"[{"foo":"bar"}]"#);
    }

    #[test]
    fn it_converts_to_value() {
        let documents: CosmosDbDocuments = json!({ "foo": "bar" }).into();
        assert_eq!(documents.len(), 1);
        assert_eq!(documents[0]["foo"].as_str().unwrap(), "bar");

        let value: Value = documents.into();
        assert!(value.is_array());
        assert_eq!(value.as_array().unwrap()[0]["foo"].as_str().unwrap(), "bar");
    }

    #[test]
    fn it_converts_to_body() {
        let documents: CosmosDbDocuments = r#"{ "foo": "bar" }"#.into();
        let body: Body = documents.into();
        assert_eq!(body.as_str().unwrap(), r#"[{"foo":"bar"}]"#);

        let documents: CosmosDbDocuments = json!({"hello": "world"}).into();
        let body: Body = documents.into();
        assert_eq!(body.as_str().unwrap(), r#"[{"hello":"world"}]"#);
    }

    #[test]
    fn it_converts_from_typed_data() {
        let mut data = protocol::TypedData::new();
        data.set_json(r#"{ "foo": "bar" }"#.to_string());

        let documents: CosmosDbDocuments = data.into();
        assert_eq!(documents.len(), 1);
        assert_eq!(documents[0]["foo"].as_str().unwrap(), "bar");
    }

    #[test]
    fn it_converts_to_typed_data() {
        let documents: CosmosDbDocuments = json!({ "foo": "bar" }).into();
        assert_eq!(documents.len(), 1);
        assert_eq!(documents[0]["foo"].as_str().unwrap(), "bar");

        let data: protocol::TypedData = documents.into();
        assert!(data.has_json());
        assert_eq!(data.get_json(), r#"[{"foo":"bar"}]"#);
    }
}
