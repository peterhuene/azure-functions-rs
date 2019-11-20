use azure_functions::{
    bindings::{HttpRequest, HttpResponse, TwilioSmsMessage},
    func,
};

#[func]
#[binding(name = "output1", from = "+15555555555")]
pub fn send_sms(mut req: HttpRequest) -> (HttpResponse, TwilioSmsMessage) {
    (
        "Text message sent.".into(),
        TwilioSmsMessage {
            to: req
                .query_params
                .remove("to")
                .expect("expected a 'to' query parameter"),
            body: req.query_params.remove("body"),
            ..Default::default()
        },
    )
}
