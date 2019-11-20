use crate::{generic::Value, http::Body, rpc::TypedData};

/// Represents a generic input binding.
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
/// An example of using a `GenericInput` binding instead of a `CosmosDbDocument` binding:
///
/// ```rust
/// use azure_functions::{
///     bindings::{GenericInput, HttpRequest, HttpResponse},
///     func,
///     generic::Value,
/// };
/// use serde_json::from_str;
///
/// #[func]
/// pub fn read_document(
///     #[binding(route = "read/{id}")] req: HttpRequest,
///     #[binding(
///         type = "cosmosDB",
///         connectionStringSetting = "connection",
///         databaseName = "exampledb",
///         collectionName = "documents",
///         id = "{id}",
///         partitionKey = "{id}"
///     )]
///     document: GenericInput,
/// ) -> HttpResponse {
///     match document.data {
///         Value::String(s) => {
///             let v: serde_json::Value = from_str(&s).expect("expected JSON data");
///             if v.is_null() {
///                 format!(
///                     "Document with id '{}' does not exist.",
///                     req.route_params.get("id").unwrap()
///                 )
///                 .into()
///             } else {
///                 v.into()
///             }
///         }
///         _ => panic!("expected string for CosmosDB document data"),
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct GenericInput {
    /// The input binding data.
    pub data: Value,
}

impl Into<Body> for GenericInput {
    fn into(self) -> Body {
        self.data.into()
    }
}

#[doc(hidden)]
impl From<TypedData> for GenericInput {
    fn from(data: TypedData) -> Self {
        Self { data: data.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::typed_data::Data;
    use serde_json::json;

    #[test]
    fn it_converts_from_typed_data() {
        let binding: GenericInput = TypedData {
            data: Some(Data::Json(r#"{ "foo": "bar" }"#.to_string())),
        }
        .into();

        assert_eq!(binding.data, Value::Json(json!({ "foo": "bar" })));
    }
}
