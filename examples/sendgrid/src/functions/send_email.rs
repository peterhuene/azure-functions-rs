use azure_functions::{
    bindings::{HttpRequest, HttpResponse, SendGridMessage},
    func,
};

#[func]
#[binding(name = "output1", from = "azure.functions.for.rust@example.com")]
pub fn send_email(req: HttpRequest) -> (HttpResponse, SendGridMessage) {
    let params = req.query_params();

    (
        "The email was sent.".into(),
        SendGridMessage::build()
            .to(params.get("to").unwrap().as_str())
            .subject(params.get("subject").unwrap().as_str())
            .content(params.get("content").unwrap().as_str())
            .finish(),
    )
}
