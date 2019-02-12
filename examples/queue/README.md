# Example Queue Triggered Azure Function

This project is an example of a simple queue-triggered Azure Function.

## Example function implementations

An example queue-triggered Azure Function that runs when a new message is posted
to the `test` Azure Storage Queue.

```rust
use azure_functions::{bindings::QueueTrigger, func};

#[func]
#[binding(name = "trigger", queue_name = "test")]
pub fn queue(trigger: &QueueTrigger) {
    log::info!("Message: {}", trigger.message);
}
```

An example queue-triggered Azure Function that outputs a message to another storage queue:

```rust
use azure_functions::{
    bindings::{QueueMessage, QueueTrigger},
    func,
};

#[func]
#[binding(name = "trigger", queue_name = "echo-in")]
#[binding(name = "$return", queue_name = "echo-out")]
pub fn queue_with_output(trigger: &QueueTrigger) -> QueueMessage {
    log::info!("Message: {}", trigger.message);

    trigger.message.clone()
}
```

# Running the example locally

Because this example relies on Azure Storage to function, the `AzureWebJobsStorage` environment
variable must be set to a connection string that the Azure Functions Host will use for the default
storage connection.

To run with the `AzureWebJobsStorage` environment variable set:

```bash
$ AzureWebJobsStorage="<insert connection string here>" cargo func run
```

_Note: the syntax above works on macOS and Linux; on Windows, set the environment variables before running `cargo func run`._

# Invoking the functions

## Invoke the `queue` function

To invoke the queue function from this example, create an Azure Storage Queue named `test` for the Azure Storage account
that was used when running the application.

Post a `hello world!` message to the queue using the Azure Portal.

With any luck, you should see the following output from the Azure Functions Host:

```
info: Function.queue[0]
      => System.Collections.Generic.Dictionary`2[System.String,System.Object]
      Executing 'Functions.queue' (Reason='New queue message detected on 'test'.', Id=01912ed1-83aa-4ac7-ae2a-9b2b1ae80830)
info: Worker.Rust.30489a08-ea06-4e63-b87b-686680a387c7[0]
      Message: hello world!
info: Function.queue[0]
      => System.Collections.Generic.Dictionary`2[System.String,System.Object]
      Executed 'Functions.queue' (Succeeded, Id=01912ed1-83aa-4ac7-ae2a-9b2b1ae80830)
```

## Invoke the `queue_with_output` function

To invoke the `queue_with_output` function, post a message to the `echo-in` queue using the Azure Portal.

After the function invokes, you should see the same message posted back to the `echo-out` queue.