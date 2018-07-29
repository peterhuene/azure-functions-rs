# Example Blob Triggered Azure Functions

This package is an example of simple blob-triggered Azure Functions.

## Example function implementations

An example blob-triggered Azure Function that runs when a new blob is created 
in the `test` Azure Storage blob container.

```rust
use azure_functions::bindings::BlobTrigger;
use azure_functions::func;

#[func]
#[binding(name = "trigger", path = "test/{name}")]
pub fn print_blob(trigger: &BlobTrigger) {
    info!("Blob (as string): {:?}", trigger.blob().as_str());
}
```

# Running the example locally

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

### Azure Functions Host

Clone the Azure Functions Host from GitHub:

```
git clone git@github.com:azure/azure-functions-host.git
```

Use `dotnet` to build the Azure Functions Host:

```
cd azure-functions-host/src/WebJobs.Script.WebHost
dotnet build
```

## Register the Rust language worker

The Azure Functions Host uses JSON configuration files to register language workers.

Create the configuration file to register the Rust language worker:

```
mkdir azure-functions-host/src/WebJobs.Script.WebHost/bin/Debug/netcoreapp2.1/workers/rust
cp azure-functions-rs/azure-functions/worker.config.json azure-functions-host/src/WebJobs.Script.WebHost/bin/Debug/netcoreapp2.1/workers/rust
```

## Initialize the example application

Run the following command to build and initialize the Rust Azure Functions application:

```
cd azure-functions-rs/examples/blob
cargo run --release -- init --worker-path /tmp/blob-example/rust_worker --script-root /tmp/blob-example/root
```

## Start the Azure Functions Host

Run the following commands to start the Azure Functions Host:

```
cd azure-functions-host/src/WebJobs.Script.WebHost
PATH=/tmp/blob-example:$PATH AzureWebJobsScriptRoot=/tmp/blob-example/root AzureWebJobsStorage=$CONNECTION_STRING dotnet run
```

Where `$CONNECTION_STRING` is the Azure Storage connection string the Azure Functions host should use.

_Note: the syntax above works on macOS and Linux; on Windows, set the environment variables before running `dotnet run`._

## Invoke the `print_blob` function

To invoke the `print_blob` function, upload a file containing the string `hello world` to blob container named `test` for the Azure Storage account
that was used for the `$CONNECTION_STRING` variable above.

With any luck, you should see the following output from the Azure Functions Host:

```
info: Function.print_blob[0]
      => System.Collections.Generic.Dictionary`2[System.String,System.Object]
      Executing 'Functions.print_blob' (Reason='New blob detected: test/hello_world.txt', Id=38848d35-01cc-4854-a3cb-3e0fb74b6704)
info: Worker.Rust.97626f24-4bbf-4895-a9d0-4362ea1e9498[0]
      Blob contents: Some("hello world")
info: Function.print_blob[0]
      => System.Collections.Generic.Dictionary`2[System.String,System.Object]
      Executed 'Functions.print_blob' (Succeeded, Id=38848d35-01cc-4854-a3cb-3e0fb74b6704)
```
