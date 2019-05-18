use azure_functions::{
    bindings::{HttpRequest, HttpResponse, TwilioSmsMessage},
    func,
};
use std::borrow::ToOwned;

#[func]
#[binding(name = "output1", from = "+15555555555")]
pub fn send_sms(req: HttpRequest) -> (HttpResponse, TwilioSmsMessage) {
    let params = req.query_params();

    (
        "Text message sent.".into(),
        TwilioSmsMessage {
            to: params.get("to").unwrap().to_owned(),
            body: params.get("body").map(ToOwned::to_owned),
            ..Default::default()
        },
    )
}
