use azure_functions::{
    bindings::{HttpRequest, TwilioSmsMessage},
    func,
};
use std::borrow::ToOwned;

#[func]
#[binding(name = "$return", from = "+15555555555")]
pub fn send_sms(req: HttpRequest) -> TwilioSmsMessage {
    let params = req.query_params();

    TwilioSmsMessage {
        to: params.get("to").unwrap().to_owned(),
        body: params.get("body").map(ToOwned::to_owned),
        ..Default::default()
    }
}
