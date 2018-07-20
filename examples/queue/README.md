# Example Queue Triggered Azure Function

This package is an example of a simple queue-triggered Azure Function.

## Example function implementations

An example queue-triggered Azure Function that runs when a new message is posted
to the `test` Azure Storage Queue.

```rust
use azure_functions::bindings::QueueTrigger;
use azure_functions::func;

#[func]
#[binding(name = "trigger", queue_name = "test")]
pub fn queue(trigger: &QueueTrigger) {
    info!("Message: {}", trigger.message());
}
```

An example queue-triggered Azure Function that outputs a message to another storage queue:

```rust
use azure_functions::bindings::{QueueMessage, QueueTrigger};
use azure_functions::func;

#[func]
#[binding(name = "trigger", queue_name = "echo-in")]
#[binding(name = "$return", queue_name = "echo-out")]
pub fn queue_with_output(trigger: &QueueTrigger) -> QueueMessage {
    let message = trigger.message();

    info!("Message: {}", message);

    message.into()
}
```

# Running the example

## Prerequisites

### Nightly Rust compiler

This example requires the use of a nightly Rust compiler due the use of the experimental procedural macros feature.

Use [rustup](https://github.com/rust-lang-nursery/rustup.rs) to install a nightly compiler:

```
rustup install nightly
rustup default nightly
```

### .NET Core SDK

The Azure Functions Host is implemented with .NET Core, so download and install a [.NET Core SDK](https://www.microsoft.com/net/download).

### Custom fork of Azure Functions Host

Currently, the Azure Functions Host does not support the Rust language worker.  Until that time, Azure Functions written in Rust must be executed locally using a [fork of the Azure Functions Host that does](https://github.com/peterhuene/azure-functions-host/tree/rust-worker-provider).

Run the following command to clone the fork:

```
git clone -b rust-worker-provider git@github.com:peterhuene/azure-functions-host.git
```

## Create the script root

Run the following command to create the "script root" for the example:

```
cargo run -q -- --create root
```

This will build and run the sample to create the "script root" containing the Rust worker and the example Azure Function metadata.

Remember the path to the root directory from this step as it will be needed for running the Azure Functions Host below.

## Start the Azure Functions Host

Run the following commands to start the Azure Functions Host:

```
cd azure-functions-host/src/WebJobs.Script.WebHost
AzureWebJobsScriptRoot=$SCRIPT_ROOT AzureWebJobsStorage=$CONNECTION_STRING dotnet run
```

Where `$SCRIPT_ROOT` above represents the path to the root directory created from running `cargo run` above and `$CONNECTION_STRING` is the Azure Storage connection string the Azure Functions host should use.

_Note: the syntax above works on macOS and Linux; on Windows, set the environment variables before running `dotnet run`._

## Invoke the `queue` function

To invoke the queue function from this example, create an Azure Storage Queue named `test` for the Azure Storage account
that was used for the `$CONNECTION_STRING` variable above.

Post a `hello world!` message to the queue.

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

Likewise, to invoke the `queue_with_output` function, post a message to the `echo-in` queue.  After the function invokes,
you should see the same message posted back to the `echo-out` queue.