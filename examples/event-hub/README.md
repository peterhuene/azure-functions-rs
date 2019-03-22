# Example Event Hub Azure Functions

This project is an example of using Event Hub with Azure Functions for Rust.

## Example function implementations

An example Event Hub triggered Azure Function that runs when a new message is posted
to the `example` Event Hub:

```rust
use azure_functions::{
    bindings::EventHubTrigger,
    func,
};

#[func]
#[binding(name = "trigger", connection = "connection", event_hub_name = "example")]
pub fn log_event(trigger: EventHubTrigger) {
    log::warn!("Event hub message: {}", trigger.message.as_str().unwrap());
}
```

An example HTTP-triggered Azure Function that outputs a message to the `example` Event Hub:

```rust
use azure_functions::{
    bindings::{HttpRequest, HttpResponse, EventHubMessage},
    func,
};

#[func]
#[binding(name = "output1", connection = "connection", event_hub_name = "example")]
pub fn create_event(_req: HttpRequest) -> (HttpResponse, EventHubMessage) {
    (
        "Created Event Hub message.".into(),
        "Hello from Rust!".into()
    )
}
```

# Running the example locally

Because this example relies on Azure Storage to function, the `AzureWebJobsStorage` setting must be set to a connection string that the Azure Functions Host will use for the default
storage connection.

Start by creating a known script root for the Azure Functions application:

```
$ cargo run -- init --script-root /tmp/myroot && cd /tmp/myroot
```

Next, add a setting for `AzureWebJobsStorage`:

```
$ func settings add AzureWebJobsStorage <storage_connection_string>
```

Additionally, this example uses a connection setting named `connection` for the Event Hubs connection string, so add that setting:

```
$ func settings add connection <event_hub_namespace_connection_string>
```

This example expects an `example` Event Hub to exist so ensure one has been created in the Azure Portal.

Finally, change back to the example directory and start the Azure Functions application:

```
$ cargo func run --script-root /tmp/myroot
```

# Invoking the functions

## Invoke the `create_event` function

This function is designed to trigger the `log_event` function by posting a message for the monitored Event Hub. 

Simply use `curl` to invoke the `create_event` function:

```
$ curl http://localhost:8080/api/create_event
```

With any luck, something like the following should be logged by the Azure Functions Host:

```
Event hub message: Hello from Rust!
```