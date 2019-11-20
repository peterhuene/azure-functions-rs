use crate::serialization::ChatMessage;
use azure_functions::{
    bindings::{HttpRequest, SignalRGroupAction},
    func,
    signalr::GroupAction,
};
use serde_json::from_slice;

#[func(name = "removeFromGroup")]
#[binding(name = "$return", hub_name = "simplechat", connection = "connection")]
pub fn remove_from_group(
    #[binding(auth_level = "anonymous", methods = "post")] req: HttpRequest,
) -> SignalRGroupAction {
    let message: ChatMessage =
        from_slice(req.body.as_bytes()).expect("failed to deserialize chat message");

    SignalRGroupAction {
        user_id: message.recipient.unwrap(),
        group_name: message.group_name.unwrap(),
        action: GroupAction::Remove,
    }
}
