# Example Table Azure Functions

This project is an example of simple table-related Azure Functions.

## Example function implementations

An example function that creates a row in an Azure Storage using an output table binding:

```rust
use azure_functions::bindings::{HttpRequest, Table};
use azure_functions::func;
use serde_json::Value;

#[func]
#[binding(
    name = "req",
    auth_level = "anonymous",
    route = "create/{table}/{partition}/{row}"
)]
#[binding(name = "output1", table_name = "{table}")]
pub fn create_row(req: &HttpRequest) -> ((), Table) {
    let mut table = Table::new();
    {
        let row = table.add_row(
            req.route_params().get("partition").unwrap(),
            req.route_params().get("row").unwrap(),
        );

        row.insert(
            "body".to_string(),
            Value::String(req.body().as_str().unwrap().to_owned()),
        );
    }
    ((), table)
}
```

An example function that reads a row using an input table binding:

```rust
use azure_functions::bindings::{HttpRequest, HttpResponse, Table};
use azure_functions::func;
use serde_json::Value;

#[func]
#[binding(
    name = "_req",
    auth_level = "anonymous",
    route = "read/{table}/{partition}/{row}"
)]
#[binding(
    name = "table",
    table_name = "{table}",
    partition_key = "{partition}",
    row_key = "{row}"
)]
pub fn read_row(_req: &HttpRequest, table: &Table) -> HttpResponse {
    table.as_value().get(0).unwrap_or(&Value::Null).into()
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
cd azure-functions-rs/examples/table
cargo run --release -- init --worker-path /tmp/table-example/rust_worker --script-root /tmp/table-example/root
```

## Start the Azure Functions Host

Run the following commands to start the Azure Functions Host:

```
cd azure-functions-host/src/WebJobs.Script.WebHost
PATH=/tmp/table-example:$PATH AzureWebJobsScriptRoot=/tmp/table-example/root AzureWebJobsStorage=$CONNECTION_STRING dotnet run
```

Where `$CONNECTION_STRING` is the Azure Storage connection string the Azure Functions host should use.

_Note: the syntax above works on macOS and Linux; on Windows, set the environment variables before running `dotnet run`._

## Invoke the `create_row` function

To create a row in a table named `test` with partition key `partition1` and row key `row1`,
use curl to invoke the `create_row` function:

```
curl -d "hello world!" http://localhost:5000/api/create/test/partition1/row1 -v
```

With any luck, this should return a `204 No Content` response.

## Invoke the `read_row` function

To read a row from a table named `test` with partition key `partition1` and row key `row1`:

```
curl http://localhost:5000/api/read/test/partition1/row1
```

With any luck, the entity should be printed by `curl`.
