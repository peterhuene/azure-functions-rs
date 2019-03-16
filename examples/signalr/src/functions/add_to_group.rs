use crate::serialization::ChatMessage;
use azure_functions::{
    bindings::{HttpRequest, SignalRGroupAction},
    func,
    signalr::GroupAction,
};

#[func(name = "addToGroup")]
#[binding(name = "req", auth_level = "anonymous", methods = "post")]
#[binding(name = "$return", hub_name = "simplechat", connection = "connection")]
pub fn add_to_group(req: HttpRequest) -> SignalRGroupAction {
    let message: ChatMessage = req
        .body()
        .as_json()
        .expect("failed to deserialize chat message");
    SignalRGroupAction {
        user_id: message.recipient.unwrap(),
        group_name: message.group_name.unwrap(),
        action: GroupAction::Add,
    }
}
