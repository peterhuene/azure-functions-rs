# Example SendGrid Azure Function

This project is an example of using SendGrid with Azure Functions for Rust.

## Example function implementation

An example HTTP-triggered Azure Function that outputs a SendGrid email message:

```rust
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
```

# Running the example locally

This example requires a [SendGrid](https://sendgrid.com/) account to run.

First, sign up for a free account with SendGrid.  The free trial account will
allow you to send a limited number of email messages .  With a paid account,
you'll be able to send a higher volume of messages.

You will need to [create a SendGrid API key](https://sendgrid.com/docs/ui/account-and-settings/api-keys/#creating-an-api-key) to use the example.

Add the SendGrid API key as the `AzureWebJobsSendGridApiKey` setting to `local.settings.json`:

```
$ func settings add AzureWebJobsSendGridApiKey <key>
```

You may encrypt `local.settings.json`, if desired:

```
$ func settings encrypt
```

Finally, start the Azure Functions application:

```
$ cargo func run
```

# Invoking the function

## Invoke the `send_email` function

Simply use `curl` to invoke the `send_email` function with the desired phone number and message body:

```
$ curl "http://localhost:8080/api/send_email?to=$EMAIL&subject=test&content=hello%20world"
```

Where `$EMAIL` is replaced with the email address you would like to send the message to.

With any luck, you should receive a "hello world" text message with the given message body.

Because the "from" address of the email is an `example.com` address, it is likely that the email
will end up in your spam folder, so make sure to check there in case it does not appear in your inbox.