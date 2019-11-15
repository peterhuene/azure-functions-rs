# Example Event Hub Azure Functions

This project is an example of using Event Hub with Azure Functions for Rust.

## Example function implementations

An example Event Hub triggered Azure Function that runs when a new message is posted
to the `example` Event Hub:

```rust
use azure_functions::{bindings::EventHubTrigger, func};

#[func]
pub fn log_event(
    #[binding(connection = "connection", event_hub_name = "example")] trigger: EventHubTrigger,
) {
    log::info!("Event hub message: {}", trigger.message.to_str().unwrap());
}
```

An example HTTP-triggered Azure Function that outputs a message to the `example` Event Hub:

```rust
use azure_functions::{
    bindings::{EventHubMessage, HttpRequest, HttpResponse},
    func,
};

#[func]
#[binding(
    name = "output1",
    connection = "connection",
    event_hub_name = "example"
)]
pub fn create_event(_req: HttpRequest) -> (HttpResponse, EventHubMessage) {
    (
        "Created Event Hub message.".into(),
        "Hello from Rust!".into(),
    )
}
```

# Running the example locally

Because this example relies on Azure Storage to function, the `AzureWebJobsStorage` setting must be set to a connection string that the Azure Functions Host will use for the default
storage connection.

Add a setting for `AzureWebJobsStorage` into `local.settings.json`:

```
$ func settings add AzureWebJobsStorage <storage_connection_string>
```

Additionally, this example uses a connection setting named `connection` for the Event Hubs connection string, so add that setting:

```
$ func settings add connection <event_hub_namespace_connection_string>
```

You may encrypt `local.settings.json`, if desired:

```
$ func settings encrypt
```

This example expects an `example` Event Hub to exist so ensure one has been created in the Azure Portal.

Finally, start the Azure Functions application:

```
$ cargo func run
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