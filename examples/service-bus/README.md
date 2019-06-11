# Example Service Bus Triggered Azure Function

This project is an example of using Service Bus with Azure Functions for Rust.

## Example function implementations

An example Service Bus triggered Azure Function that runs when a new message is posted
to the `example` queue:

```rust
use azure_functions::{bindings::ServiceBusTrigger, func};

#[func]
#[binding(name = "trigger", queue_name = "example", connection = "connection")]
pub fn log_queue_message(trigger: ServiceBusTrigger) {
    log::info!("{}", trigger.message.as_str().unwrap());
}
```

An example HTTP-triggered Azure Function that outputs a Service Bus message to the `example` queue:

```rust
use azure_functions::{
    bindings::{HttpRequest, ServiceBusMessage},
    func,
};

#[func]
#[binding(name = "$return", queue_name = "example", connection = "connection")]
pub fn create_queue_message(req: HttpRequest) -> ServiceBusMessage {
    format!(
        "Hello from Rust, {}!\n",
        req.query_params().get("name").map_or("stranger", |x| x)
    )
    .into()
}
```

An example Service Bus triggered Azure Function that runs when a new message is posted
to the `mytopic` topic and `mysubscription` subscription:

```rust
use azure_functions::{bindings::ServiceBusTrigger, func};

#[func]
#[binding(
    name = "trigger",
    topic_name = "mytopic",
    subscription_name = "mysubscription",
    connection = "connection"
)]
pub fn log_topic_message(trigger: ServiceBusTrigger) {
    log::info!("{}", trigger.message.as_str().unwrap());
}
```

An example HTTP-triggered Azure Function that outputs a Service Bus message to the `mytopic` topic and `mysubscription` subscription:

```rust
use azure_functions::{
    bindings::{HttpRequest, ServiceBusMessage},
    func,
};

#[func]
#[binding(
    name = "$return",
    topic_name = "mytopic",
    subscription_name = "mysubscription",
    connection = "connection"
)]
pub fn create_topic_message(req: HttpRequest) -> ServiceBusMessage {
    format!(
        "Hello from Rust, {}!\n",
        req.query_params().get("name").map_or("stranger", |x| x)
    )
    .into()
}
```

# Running the example locally

Because this example relies on Azure Storage to function, the `AzureWebJobsStorage`
setting must be set to a connection string that the Azure Functions Host will use for 
the default storage connection.

Add a setting for `AzureWebJobsStorage` into `local.settings.json`:

```
$ func settings add AzureWebJobsStorage <storage_connection_string>
```

Additionally, this example uses a connection setting named `connection` for the Service Bus connection string, so add that setting:

```
$ func settings add connection <service_bus_connection_string>
```

You may encrypt `local.settings.json`, if desired:

```
$ func settings encrypt
```

This example expects a Service Bus queue named `example` to exist.  Use the Azure Portal to create a Service Bus queue with that name.

Additionally, this example expects a Service Bus topic named `mytopic` to have a subscription named `mysubscription`.  Use the Azure Portal to create a Service Bus topic and subscription with those names.

Finally, start the Azure Functions application:

```
$ cargo func run
```

# Invoking the functions

## Invoke the `create_queue_message` function

This function is designed to trigger the `log_queue_message` function by creating a Service Bus message in the queue being monitored.

Simply use `curl` to invoke the `create_queue_message` function with the desired document identifier:

```
$ curl http://localhost:8080/api/create_queue_message
```

With any luck, you should see the following informational message printed to your terminal:

```
[3/26/19 5:29:34 AM] Hello from Rust, stranger!
```

## Invoke the `create_topic_message` function

This function is designed to trigger the `log_topic_message` function by creating a Service Bus message in the topic being monitored.

Simply use `curl` to invoke the `create_topic_message` function with the desired document identifier:

```
$ curl http://localhost:8080/api/create_topic_message\?name=Peter
```

With any luck, you should see the following informational message printed to your terminal:

```
[3/26/19 5:30:29 AM] Hello from Rust, Peter!
```