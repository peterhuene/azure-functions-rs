# Azure Functions In Rust

[![crates.io](https://img.shields.io/crates/v/azure-functions.svg)](https://crates.io/crates/azure-functions)
[![docs.rs](https://docs.rs/azure-functions/badge.svg)](https://docs.rs/azure-functions)
[![license](https://img.shields.io/crates/l/azure-functions.svg)](https://github.com/peterhuene/azure-functions-rs/blob/master/LICENSE)

This is an early-stage prototype for implementing [Azure Functions](https://azure.microsoft.com/en-us/services/functions/)
in [Rust](https://www.rust-lang.org/en-US/).

***Disclaimer: althougth the maintainer of this repository is a Microsoft employee, this project is not an official Microsoft product
and is not an endorsement of any future product offering from Microsoft.***

***This is simply a labor of love by a developer who would like to see Rust adoption flourish.***

## Examples

An example anonymous, HTTP-triggered Azure Function:

```rust
use azure_functions::bindings::{HttpRequest, HttpResponse};
use azure_functions::func;

#[func]
#[binding(name = "req", auth_level = "anonymous")]
pub fn greet(req: &HttpRequest) -> HttpResponse {
    info!("The request was: {:?}", req);

    format!(
        "Hello from Rust, {}!\n",
        req.query_params().get("name").map_or("stranger", |x| x)
    ).into()
}
```

A timer-triggered Azure Function that is invoked every 5 minutes:

```rust
use azure_functions::bindings::TimerInfo;
use azure_functions::func;

#[func]
#[binding(name = "info", schedule = "0 */5 * * * *")]
pub fn timer(info: &TimerInfo) {
    info!("Rust function invoked by timer!");
}
```

_Note: the `info!` macro used in the above examples comes from the [log crate](https://crates.io/crates/log); the messages will be logged on the Azure Functions Host._

See the [examples](https://github.com/peterhuene/azure-functions-rs/tree/master/examples) directory for the complete source code.

# Documentation

Documentation for the [latest published version](https://docs.rs/azure-functions).

# Azure Functions Bindings

Azure Functions supports a [wide variety of input and output bindings](https://docs.microsoft.com/en-us/azure/azure-functions/functions-triggers-bindings) that can be used by a function.

In a language like C#, parameters can be annotated with attributes describing how the parameters are bound.

Rust does not support attributes on parameters, so the `#[binding]` attribute is applied on the function with a name that matches the parameter's identifier.  The arguments to the attribute depend on the binding type.

The `#[func]` attribute is used to turn an ordinary Rust function into an Azure Function.  It works by examining the parameters and return type to the function and automatically mapping them to corresponding bindings.

The current list of supported bindings:

| Rust Type                                 | Azure Functions Binding |
|-------------------------------------------|-------------------------|
| `azure_functions::bindings::HttpRequest`  | HTTP Trigger            |
| `azure_functions::bindings::HttpResponse` | HTTP Output             |
| `azure_functions::bindings::TimerInfo`    | Timer Trigger           |
| `azure_functions::Context`*               | Invocation Context      |

\****Note: the `Context` binding is not an Azure Functions binding; it is used to pass information about the function being invoked.***

More bindings will be implemented in the future, including support for retreiving data from custom bindings.

# Development

## Cloning the Repository

This repository uses a git submodule for defining the [Azure Functions Language Worker Protocol](https://github.com/Azure/azure-functions-language-worker-protobuf).

Use `--recurse-submodules` when cloning this repository:

```
git clone --recurse-submodules git@github.com:peterhuene/azure-functions-rs.git
```

## Repository Layout

This repository is split into multiple Rust crates:

* [codegen](https://github.com/peterhuene/azure-functions-rs/tree/master/codegen) - The `azure-functions-codegen` crate that defines the procedural macros that are used when writing Azure Functions in Rust.  The generated code is used to build a Azure Functions Worker for Rust.
* [lib](https://github.com/peterhuene/azure-functions-rs/tree/master/lib) - The `azure-functions` crate that defines the types and functions that are used by Azure Functions written in Rust.
    * Note: the `lib/protobuf` directory is the git submodule for [Azure Functions Language Worker Protocol](https://github.com/Azure/azure-functions-language-worker-protobuf).
* [examples/http](https://github.com/peterhuene/azure-functions-rs/tree/master/examples/http) - An example of an HTTP-triggered function.
* [examples/timer](https://github.com/peterhuene/azure-functions-rs/tree/master/examples/timer) - An example of a timer-triggered function.

## Prerequisites

### Nightly Rust Compiler

This repository requires the use of a nightly Rust compiler due the use of the experimental procedural macros feature.

Use [rustup](https://github.com/rust-lang-nursery/rustup.rs) to install a nightly compiler:

```
rustup install nightly
rustup default nightly
```

### Google Protocol Buffers Compiler

The `azure-functions` crate depends on the [protoc-grpcio](https://github.com/mtp401/protoc-grpcio) crate to generate Rust code for the Azure Functions Language Worker protocol definitions.

Therefore, Google's Protocol Buffers compiler (`protoc`) must be installed and on the PATH to build `azure-functions`.  See the [Protocol Buffer repository](https://github.com/google/protobuf) for information on how to install the compiler.

## Building

Build at the root of the repository to build both the `azure-functions-codegen` and the `azure-functions` libraries using `cargo`:

```
cargo build
```

## Running tests

Use `cargo` to run the tests:

```
cargo test
```

Right now there are only doc tests, but more tests are coming soon.

## Running the examples locally

Currently, the Azure Functions Host does not support the Rust language worker.  Until that time, Azure Functions written in Rust must be executed locally using a [fork of the Azure Functions Host that does](https://github.com/peterhuene/azure-functions-host/tree/rust-worker-provider).

### Run the HTTP example

Run the following commands:

```
cd examples/http
cargo run -q -- --create root
```

This will build and run the sample to create the Azure Functions "script root" containing the Rust worker and the example Azure Function metadata.

Remember the path to the root directory from this step, it will be needed for running the Azure Functions Host (see below).

### Download a .NET SDK

The Azure Functions Host is implemented with .NET Core, so download and install a [.NET Core SDK](https://www.microsoft.com/net/download).

### Clone the fork of the Azure Functions Host

Run the following command to clone the fork:

```
git clone -b rust-worker-provider git@github.com:peterhuene/azure-functions-host.git
```

### Run the Azure Functions Host

Run the following commands to run the Azure Functions Host locally:

```
cd azure-functions-host/src/WebJobs.Script.WebHost
AzureWebJobsScriptRoot=$SCRIPT_ROOT_PATH dotnet run
```

Where `$SCRIPT_ROOT_PATH` above represents the path to the root directory created from running `cargo run` above.

_Note: the syntax above works on macOS and Linux; on Windows, set the `AzureWebJobsScriptRoot` environment variable before running `dotnet run`._

_Note: if using bindings that require storage (such as timer triggers), you must set the `AzureWebJobsStorage` environment variable to an Azure Storage connection string._

### Invoke the `greet` function

The easiest way to invoke the function is to use `curl` (substitute for your preferred method of sending HTTP requests):

```
curl localhost:5000/api/greet\?name=Peter
```

With any luck, you should see the following output:

```
Hello from Rust, Peter!
```
