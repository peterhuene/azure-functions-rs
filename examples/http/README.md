# Example HTTP Azure Function

This project is an example of a simple HTTP-triggered Azure Function.

## Example function implementations

An example HTTP-triggered Azure Function:

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

An example HTTP-triggered Azure Function using JSON for request and response:

```rust
use azure_functions::bindings::{HttpRequest, HttpResponse};
use azure_functions::func;
use azure_functions::http::Status;
use serde_json::to_value;

#[derive(Deserialize)]
struct Request {
    name: String,
}

#[derive(Serialize)]
struct Response {
    message: String,
}

#[func]
#[binding(name = "req", auth_level = "anonymous")]
pub fn greet_with_json(req: &HttpRequest) -> HttpResponse {
    if let Ok(request) = req.body().from_json::<Request>() {
        let response = Response {
            message: format!("Hello from Rust, {}!", request.name),
        };
        return to_value(response).unwrap().into();
    }

    HttpResponse::build()
        .status(Status::BadRequest)
        .body("Invalid JSON request.")
        .into()
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
cd azure-functions-rs/examples/http
cargo run --release -- init --worker-path /tmp/http-example/rust_worker --script-root /tmp/http-example/root
```

## Start the Azure Functions Host

Run the following commands to start the Azure Functions Host:

```
cd azure-functions-host/src/WebJobs.Script.WebHost
PATH=/tmp/http-example:$PATH AzureWebJobsScriptRoot=/tmp/http-example/root dotnet run
```

_Note: the syntax above works on macOS and Linux; on Windows, set the `AzureWebJobsScriptRoot` environment variable before running `dotnet run`._

## Invoke the `greet` function

The easiest way to invoke the function is to use `curl`:

```
curl localhost:5000/api/greet\?name=Peter
```

With any luck, you should see the following output:

```
Hello from Rust, Peter!
```

## Invoke the `greet_with_json` function

The easiest way to invoke the function is to use `curl`:

```
curl --header "Content-Type: application/json" -d '{"name": "Peter"}' http://localhost:5000/api/greet_with_json
```

With any luck, you should see the following output:

```json
{
  "message": "Hello from Rust, Peter!"
}
```