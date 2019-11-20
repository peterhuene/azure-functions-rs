use crate::{
    rpc::{typed_data::Data, TypedData},
    FromVec,
};
use serde::{Deserialize, Serialize};
use serde_json::{to_string, to_value, Value};

/// Represents the SignalR message output binding.
///
/// The following binding attributes are supported:
///
/// | Name         | Description                                                                                                                  |
/// |--------------|------------------------------------------------------------------------------------------------------------------------------|
/// | `name`       | The name of the parameter being bound.                                                                                       |
/// | `hub_name`   | The name of the SignalR hub that will receive the message.                                                                   |
/// | `connection` | The name of the app setting that contains the SignalR Service connection string. Defaults to `AzureSignalRConnectionString`. |
///
/// # Examples
///
/// This example implements an HTTP-triggered Azure Function that returns a SignalRMessage binding:
///
/// ```rust
/// use azure_functions::{
///     bindings::{HttpRequest, SignalRMessage},
///     func,
/// };
/// use serde_json::{to_value, Value};
///
/// #[func]
/// #[binding(name = "$return", hub_name = "chat", connection = "myconnection")]
/// pub fn send_message(
///     #[binding(auth_level = "anonymous", methods = "post")] mut req: HttpRequest
/// ) -> SignalRMessage {
///     SignalRMessage {
///         user_id: req.query_params.remove("user"),
///         group_name: req.query_params.remove("group"),
///         target: "newMessage".to_owned(),
///         arguments: vec![req.query_params.remove("message").map_or(Value::Null, |v| to_value(v).unwrap())],
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalRMessage {
    /// The optional user id to send the message to.
    pub user_id: Option<String>,
    /// The optional group name to send the message to.
    pub group_name: Option<String>,
    /// The target method name to invoke on each SignalR client.
    pub target: String,
    /// The arguments to pass to the target method.
    pub arguments: Vec<Value>,
}

#[doc(hidden)]
impl Into<TypedData> for SignalRMessage {
    fn into(self) -> TypedData {
        TypedData {
            data: Some(Data::Json(
                to_string(&self).expect("failed to convert SignalR message to JSON string"),
            )),
        }
    }
}

#[doc(hidden)]
impl FromVec<SignalRMessage> for TypedData {
    fn from_vec(vec: Vec<SignalRMessage>) -> Self {
        Self {
            data: Some(Data::Json(
                Value::Array(vec.into_iter().map(|m| to_value(m).unwrap()).collect()).to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_value;

    #[test]
    fn it_serializes_to_json() {
        let json = to_string(&SignalRMessage {
            user_id: Some("foo".to_owned()),
            group_name: Some("bar".to_owned()),
            target: "baz".to_owned(),
            arguments: vec![
                to_value(1).unwrap(),
                to_value("foo").unwrap(),
                to_value(false).unwrap(),
            ],
        })
        .unwrap();

        assert_eq!(
            json,
            r#"{"userId":"foo","groupName":"bar","target":"baz","arguments":[1,"foo",false]}"#
        );
    }

    #[test]
    fn it_converts_to_typed_data() {
        let message = SignalRMessage {
            user_id: Some("foo".to_owned()),
            group_name: Some("bar".to_owned()),
            target: "baz".to_owned(),
            arguments: vec![
                to_value(1).unwrap(),
                to_value("foo").unwrap(),
                to_value(false).unwrap(),
            ],
        };

        let data: TypedData = message.into();
        assert_eq!(
            data.data,
            Some(Data::Json(
                r#"{"userId":"foo","groupName":"bar","target":"baz","arguments":[1,"foo",false]}"#
                    .to_string()
            ))
        );
    }
}
