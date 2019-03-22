use crate::{rpc::protocol, signalr::GroupAction, FromVec};
use serde_derive::{Deserialize, Serialize};
use serde_json::{to_string, to_value, Value};

/// Represents the SignalR group action output binding.
///
/// # Examples
///
/// This example implements an HTTP-triggered Azure Function that adds a user to a group:
///
/// ```rust
/// use azure_functions::{
///     bindings::{HttpRequest, SignalRGroupAction},
///     func,
///     signalr::GroupAction,
/// };
///
/// #[func]
/// #[binding(name = "req", auth_level = "anonymous", methods = "post")]
/// #[binding(name = "$return", hub_name = "chat", connection = "myconnection")]
/// pub fn add_to_group(req: HttpRequest) -> SignalRGroupAction {
///     SignalRGroupAction {
///         user_id: req.query_params().get("user").unwrap().to_owned(),
///         group_name: req.query_params().get("group").unwrap().to_owned(),
///         action: GroupAction::Add,
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalRGroupAction {
    /// The name of the group to operate on.
    pub group_name: String,
    /// The user id to operate on.
    pub user_id: String,
    /// The action to take.
    pub action: GroupAction,
}

#[doc(hidden)]
impl Into<protocol::TypedData> for SignalRGroupAction {
    fn into(self) -> protocol::TypedData {
        let mut data = protocol::TypedData::new();
        data.set_json(
            to_string(&self).expect("failed to convert SignalR group action to JSON string"),
        );

        data
    }
}

#[doc(hidden)]
impl FromVec<SignalRGroupAction> for protocol::TypedData {
    fn from_vec(vec: Vec<SignalRGroupAction>) -> Self {
        let mut data = protocol::TypedData::new();
        data.set_json(
            Value::Array(vec.into_iter().map(|a| to_value(a).unwrap()).collect()).to_string(),
        );
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_serializes_to_json() {
        let json = to_string(&SignalRGroupAction {
            group_name: "foo".to_owned(),
            user_id: "bar".to_owned(),
            action: GroupAction::Add,
        })
        .unwrap();

        assert_eq!(json, r#"{"groupName":"foo","userId":"bar","action":"add"}"#);
    }

    #[test]
    fn it_converts_to_typed_data() {
        let action = SignalRGroupAction {
            group_name: "foo".to_owned(),
            user_id: "bar".to_owned(),
            action: GroupAction::Remove,
        };

        let data: protocol::TypedData = action.into();
        assert_eq!(
            data.get_json(),
            r#"{"groupName":"foo","userId":"bar","action":"remove"}"#
        );
    }
}
