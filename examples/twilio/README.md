# Example Twilio Azure Function

This project is an example of using Twilio with Azure Functions for Rust.

## Example function implementation

An example HTTP-triggered Azure Function that outputs a Twilio SMS message:

```rust
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
```

# Running the example locally

This example requires a [Twilio](https://www.twilio.com/) account to run.

First, sign up for a free account with Twilio.  The free trial account will
allow you to send text messages to your own registered phone number.  With a paid account,
you'll be able to send text messages to any supported phone number.

You will need three pieces of information from Twilio:

* Your Twilio account SID.
* Your Twilio auth token.
* An active Twilio phone number to send text messages from.

Add the Twilio account SID as the `AzureWebJobsTwilioAccountSid` setting to `local.settings.json`:

```
$ func settings add AzureWebJobsTwilioAccountSid <sid>
```

Add the Twilio auth token as the `AzureWebJobsTwilioAuthToken` setting to `local.settings.json`:

```
$ func settings add AzureWebJobsTwilioAuthToken <token>
```

You may encrypt `local.settings.json`, if desired:

```
$ func settings encrypt
```

Next, edit `src/functions/send_sms.rs` and replace the placeholder phone number (`+15555555555`) with
the Twilio phone number you activated:

```rust
...
#[binding(name = "$return", from = "<number>")]
...
```

Finally, start the Azure Functions application:

```
$ cargo func run
```

# Invoking the function

## Invoke the `send_sms` function

If you have a trial Twilio account, you will only be able to send an SMS message to the personal phone number you verified with your account.

Simply use `curl` to invoke the `send_sms` function with the desired phone number and message body:

```
$ curl -v "http://localhost:8080/api/send_sms?to=$NUMBER&body=hello%20world"
```

Where `$NUMBER` is replaced with the personal phone number you verified with your Twilio account.

With any luck, you should receive a "hello world" text message with the given message body.