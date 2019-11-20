use azure_functions::{
    bindings::{HttpRequest, HttpResponse, SendGridMessage},
    func,
};

#[func]
#[binding(name = "output1", from = "azure.functions.for.rust@example.com")]
pub fn send_email(mut req: HttpRequest) -> (HttpResponse, SendGridMessage) {
    (
        "The email was sent.".into(),
        SendGridMessage::build()
            .to(req
                .query_params
                .remove("to")
                .expect("expected a 'to' query parameter"))
            .subject(
                req.query_params
                    .remove("subject")
                    .expect("expected a 'subject' query parameter"),
            )
            .content(
                req.query_params
                    .remove("content")
                    .expect("expected a 'content' query parameter"),
            )
            .finish(),
    )
}
