# Example HTTP Azure Function

This package is an example of a simple HTTP-triggered Azure Function.

## Example function implementation

The example anonymous, HTTP-triggered Azure Function:

```rust
use azure_functions::bindings::{HttpRequest, HttpResponse};
use azure_functions::{func, Context};

#[func]
#[binding(name = "req", auth_level = "anonymous")]
pub fn greet(context: &Context, req: &HttpRequest) -> HttpResponse {
    info!("Context: {:?}, Request: {:?}", context, req);

    format!(
        "Hello from Rust, {}!\n",
        req.query_params().get("name").map_or("stranger", |x| x)
    ).into()
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
AzureWebJobsScriptRoot=$SCRIPT_ROOT_PATH dotnet run
```

Where `$SCRIPT_ROOT_PATH` above represents the path to the root directory created from running `cargo run` above.

_Note: the syntax above works on macOS and Linux; on Windows, set the `AzureWebJobsScriptRoot` environment variable before running `dotnet run`._

_Note: if using bindings that require storage (such as timer triggers), you must set the `AzureWebJobsStorage` environment variable to an Azure Storage connection string._

## Invoke the `greet` function

The easiest way to invoke the function is to use `curl`:

```
curl localhost:5000/api/greet\?name=Peter
```

With any luck, you should see the following output:

```
Hello from Rust, Peter!
```
