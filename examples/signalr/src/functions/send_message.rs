use crate::serialization::ChatMessage;
use azure_functions::{
    bindings::{HttpRequest, SignalRMessage},
    func,
};
use serde_json::{from_slice, to_value};

#[func(name = "messages")]
#[binding(name = "$return", hub_name = "simplechat", connection = "connection")]
pub fn send_message(
    #[binding(auth_level = "anonymous", methods = "post")] req: HttpRequest,
) -> SignalRMessage {
    let message: ChatMessage =
        from_slice(req.body.as_bytes()).expect("failed to deserialize chat message");

    SignalRMessage {
        user_id: message.recipient.clone(),
        group_name: message.group_name.clone(),
        target: "newMessage".to_string(),
        arguments: vec![to_value(message).expect("failed to serialize chat message")],
    }
}
