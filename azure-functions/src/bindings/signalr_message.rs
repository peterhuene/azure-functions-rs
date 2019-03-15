use crate::rpc::protocol;
use serde_derive::{Deserialize, Serialize};
use serde_json::{to_string, Value};

/// Represents the SignalR message output binding.
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
/// #[binding(name = "req", auth_level = "anonymous", methods = "post")]
/// #[binding(name = "$return", hub_name = "chat", connection = "myconnection")]
/// pub fn send_message(req: HttpRequest) -> SignalRMessage {
///     SignalRMessage {
///         user_id: req.query_params().get("user").map(|v| v.to_owned()),
///         group_name: req.query_params().get("group").map(|v| v.to_owned()),
///         target: "newMessage".to_owned(),
///         arguments: vec![req.query_params().get("message").map_or(Value::Null, |v| to_value(v).unwrap())],
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
impl Into<protocol::TypedData> for SignalRMessage {
    fn into(self) -> protocol::TypedData {
        let mut data = protocol::TypedData::new();
        data.set_json(to_string(&self).expect("failed to convert SignalR message to JSON string"));
        data
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

        let data: protocol::TypedData = message.into();
        assert_eq!(
            data.get_json(),
            r#"{"userId":"foo","groupName":"bar","target":"baz","arguments":[1,"foo",false]}"#
        );
    }
}
