# Azure Functions for Rust

[![crates.io](https://img.shields.io/crates/v/azure-functions.svg)](https://crates.io/crates/azure-functions)
[![docs.rs](https://docs.rs/azure-functions/badge.svg)](https://docs.rs/azure-functions)
[![Build Status](https://dev.azure.com/azure-functions-rs/Azure%20Functions%20for%20Rust/_apis/build/status/peterhuene.azure-functions-rs?branchName=master)](https://dev.azure.com/azure-functions-rs/Azure%20Functions%20for%20Rust/_build/latest?definitionId=2&branchName=master)
[![Dependabot Status](https://api.dependabot.com/badges/status?host=github&repo=peterhuene/azure-functions-rs)](https://dependabot.com)
[![license](https://img.shields.io/crates/l/azure-functions.svg)](https://github.com/peterhuene/azure-functions-rs/blob/master/LICENSE)

This is an early-stage framework for implementing [Azure Functions](https://azure.microsoft.com/en-us/services/functions/)
in [Rust](https://www.rust-lang.org/en-US/).

## Disclaimer

Although the maintainer of this project is a Microsoft employee, this project is not an officially recognized Microsoft product and is not an endorsement of any future product offering from Microsoft.

_Microsoft and Azure are registered trademarks of Microsoft Corporation._

## Example

A simple HTTP-triggered Azure Function:

```rust
use azure_functions::bindings::{HttpRequest, HttpResponse};
use azure_functions::func;

#[func]
pub fn greet(req: HttpRequest) -> HttpResponse {
    // Log the message with the Azure Functions Host
    info!("Request: {:?}", req);

    format!(
        "Hello from Rust, {}!\n",
        req.query_params().get("name").map_or("stranger", |x| x)
    ).into()
}
```

See the [examples](https://github.com/peterhuene/azure-functions-rs/tree/master/examples) directory for the complete list of examples.

# Documentation

Documentation for the [latest published version](https://docs.rs/azure-functions).

# Getting Started

## Install CMake

The `azure-functions` crate has a dependency on the `grpcio` crate that uses [CMake](https://cmake.org) to build and link against the gRPC native library.

CMake must be installed and on the `PATH` to be able to use Azure Functions for Rust.

### Windows

Install CMake from the [Windows installer](https://cmake.org/download/).

### macOS

The easiest way to install CMake on macOS is with [Homebrew](https://brew.sh/):

```
$ brew install cmake
```

### Linux

Use your distro's package manager to install a `cmake` (or similar) package.

For example on Debian/Ubuntu:

```
$ apt-get install cmake
```

## Install the Azure Functions Core Tools

Install version 2 or higher of the [Azure Functions Core Tools](https://docs.microsoft.com/en-us/azure/azure-functions/functions-run-local#install-the-azure-functions-core-tools).

If you are on Windows, you must add `%ProgramFiles%\nodejs\node_modules\azure-functions-core-tools\bin` (where `func.exe` is located) to the `PATH` environment variable.

## Installing the Azure Functions for Rust SDK

Install the Azure Functions for Rust SDK using `cargo install`:

```bash
$ cargo install azure-functions-sdk
```

This installs a new cargo command named `func` that can be used to create and run new Azure Functions applications.

## Creating a new Azure Functions application

Use the `cargo func new-app` command to create a new Azure Functions application:

```bash
$ cargo func new-app hello
```

This will create a new application in the `./hello` directory with a module named `functions` where the exported Azure Functions are expected to be placed.

## Adding a simple HTTP-triggered application

Use the `cargo func new` command to create a new HTTP-triggered Azure Function named `hello`:

```bash
$ cargo func new http -n hello
```

The source for the function will be in `src/functions/hello.rs`.

## Building the Azure Functions application

To build your Azure Functions application, just use `cargo build`:

```
$ cargo build
```

If you are using a nightly compiler, you can enable improved error messages during compilation.

Add the following to your `Cargo.toml`:

```toml
[features]
unstable = ["azure-functions/unstable"]
```

Build your application:

```
$ cargo build --features unstable
```

This enables Azure Functions for Rust to emit diagnostic messages that will include the position of an error within an attribute.

## Running the Azure Functions application

To build and run your Azure Functions application, use `cargo func run`:

```
$ cargo func run
```

The `cargo func run` command builds and runs your application locally using the Azure Function Host that was
installed by the Azure Functions Core Tools.

By default, the host will be configured to listen on `0.0.0.0:8080`.

For the `hello` function added previously, it can be invoked from `http://localhost:8080/api/hello`.

## Debugging the Azure Functions application

The easiest way to debug the Azure Functions application is to use [Visual Studio Code](https://code.visualstudio.com/) with the [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb) extension.

Copy the example [launch.json](https://github.com/peterhuene/azure-functions-rs/tree/master/examples/http/.vscode/launch.json) and
[tasks.json](https://github.com/peterhuene/azure-functions-rs/tree/master/examples/http/.vscode/tasks.json) files to the `.vscode` directory inside the root of your project.

This will enable a `Debug` launch configuration that will build and run your application locally before attaching the `lldb` debugger to the Rust worker process.

## Deploying the Azure Functions application

In the future, there will be a `cargo func deploy` command to deploy the Azure Functions application directly to Azure.

Until that time, you must manually build and push the Docker image to a repository that can be accessed by Azure.

**Note: this requires [Docker](https://www.docker.com/get-started) that is at least *18.06* for the experimental BuildKit support.**

**To enable the BuildKit support, set the `DOCKER_BUILDKIT` environment variable to `1` before running `docker build`.**

Start by building your image with `docker build -t $TAG_NAME .`:

```
$ docker build -t $TAG_NAME .
```

Where `$TAG_NAME` is the tag name to use (e.g. `username/my-functions-app`).

Use `docker push` to push the image to a repository that is accessible to Azure Functions.

```
$ docker push $TAG_NAME
```

Create the Function App in [Azure](https://portal.azure.com) using the "Linux (Preview)" OS.  Under the "Publish" setting, select "Docker Image".

From the "Configure Container" section, select the repository and enter the image you pushed.

That's it! Once the Functions App starts in Azure, you should be able to view the functions and run them.

# Azure Functions Bindings

Azure Functions supports a [wide variety of input and output bindings](https://docs.microsoft.com/en-us/azure/azure-functions/functions-triggers-bindings) that can be used by a function.

In a language like C#, parameters can be annotated with attributes describing how the parameters are bound.

Rust does not support attributes on parameters, so the `#[binding]` attribute is applied on the function with a name that matches the parameter's identifier.  The arguments to the attribute depend on the binding type.

The `#[func]` attribute is used to turn an ordinary Rust function into an Azure Function.  It works by examining the parameters and return type to the function and automatically mapping them to corresponding bindings.

The current list of supported bindings:

| Rust Type                                          | Azure Functions Binding              | Direction      | Vec\<T> |
|----------------------------------------------------|--------------------------------------|----------------|---------|
| `azure_functions::bindings::Blob`                  | Input and Ouput Blob                 | in, inout, out | No      |
| `azure_functions::bindings::BlobTrigger`           | Blob Trigger                         | in, inout      | No      |
| `azure_functions::bindings::CosmosDbTrigger`       | Cosmos DB Trigger                    | in             | No      |
| `azure_functions::bindings::CosmosDbDocument`      | Input and Output Cosmos DB Document  | in, out        | Yes     |
| `azure_functions::bindings::EventGridEvent`        | Event Grid Trigger                   | in             | No      |
| `azure_functions::bindings::EventHubTrigger`       | Event Hub Trigger                    | in             | No      |
| `azure_functions::bindings::EventHubMessage`       | Event Hub Output Message             | out            | Yes     |
| `azure_functions::bindings::HttpRequest`           | HTTP Trigger                         | in             | No      |
| `azure_functions::bindings::HttpResponse`          | Output HTTP Response                 | out            | No      |
| `azure_functions::bindings::QueueTrigger`          | Queue Trigger                        | in             | No      |
| `azure_functions::bindings::QueueMessage`          | Output Queue Message                 | out            | Yes     |
| `azure_functions::bindings::SignalRConnectionInfo` | SignalR Connection Info              | in             | No      |
| `azure_functions::bindings::SignalRGroupAction`    | SignalR Group Action                 | out            | Yes     |
| `azure_functions::bindings::SignalRMessage`        | SignalR Message                      | out            | Yes     |
| `azure_functions::bindings::Table`                 | Input and Ouput Table                | in, out        | No      |
| `azure_functions::bindings::TimerInfo`             | Timer Trigger                        | in             | No      |
| `azure_functions::Context`*                        | Invocation Context                   | N/A            | N/A     |

\****Note: the `Context` binding is not an Azure Functions binding; it is used to pass information about the function being invoked.***

More bindings will be implemented in the future, including support for retreiving data from custom bindings.

## Bindings in Rust

Azure Functions for Rust automatically infers the direction of bindings depending on how the binding is used in a function's declaration.

### Input bindings

Parameters of type `T` or `&T`, where `T` is a trigger or input binding type, are inferred to be bindings with an `in` direction.

```rust
#[func]
...
pub fn example(..., blob: Blob) {
    ...
}
```

```rust
#[func]
...
pub fn example(..., blob: &Blob) {
    ...
}
```

Additionally, some input binding types support parameters of type `Vec<T>` and `&Vec<T>` where `T` is an input binding type:

```rust
#[func]
...
pub fn example(..., documents: Vec<CosmosDbDocument>) {
    ...
}
```

The following input bindings support parameters of type `Vec<T>`:

* `CosmosDbDocument`

### Input-output (inout) bindings

Parameters of type `&mut T`, where `T` is a trigger or input binding type that supports the `inout` direction, are inferred to be bindings with an `inout` direction.

```rust
#[func]
...
pub fn example(..., blob: &mut Blob) {
    ...
}
```

**Note: `inout` direction bindings are currently not implemented for languages other than C#.**

**See [this issue](https://github.com/Azure/azure-functions-host/issues/49) regarding this problem with the Azure Functions Host.**

### Output bindings

Functions that return a type `T`, where `T` is an output binding type, or a tuple of output binding types, are inferred to be bindings with an `out` direction.  

```rust
#[func]
...
pub fn example(...) -> Blob {
    ...
}
```

Functions may also return `Option<T>` for any output binding type `T`; a `None` value will skip outputting a value.


```rust
#[func]
...
pub fn example(...) -> Option<Blob> {
    ...
}
```

```rust
#[func]
...
pub fn example(...) -> (HttpResponse, Option<Blob>) {
    ...
}
```

Additionally, some output binding types support returning  `Vec<T>` where `T` is an output binding type:

```rust
#[func]
...
pub fn example(...) -> Vec<CosmosDbDocument>) {
    ...
}
```

The following output bindings support returning type `Vec<T>`:

* `CosmosDbDocument`
* `EventHubMessage`
* `QueueMessage`
* `SignalRMessage`
* `SignalRGroupAction`

For functions that return a single output binding type, the binding has a special name of `$return`
and is treated as the return value of the function.

For functions that return a tuple of output binding types, the first value of the tuple has the binding name
of `$return` and is treated as the return value of the function.  The remaining values have binding names `output1`, `output2`, ..., `output(N-1)` where `N` is the number of types in the tuple, and are
treated as output bindings only.

Unit tuples `()` can be used in a tuple to "skip" a binding:

```rust
#[func]
...
pub fn example(...) -> ((), Blob) {
    ...
}
```

For the above example, there is no `$return` binding and the Azure Function "returns" no value.  Instead, a single output binding named `output1` is used.

# Development

## Cloning the Repository

This repository uses a git submodule for defining the [Azure Functions Language Worker Protocol](https://github.com/Azure/azure-functions-language-worker-protobuf).

Use `--recurse-submodules` when cloning this repository:

```
$ git clone --recurse-submodules git@github.com:peterhuene/azure-functions-rs.git
```

## Repository Layout

This repository is split into multiple Rust crates:

* [azure-functions](https://github.com/peterhuene/azure-functions-rs/tree/master/azure-functions) - The `azure-functions` crate that defines the types and functions that are used when writing Azure Functions with Rust.
* [azure-functions-codegen](https://github.com/peterhuene/azure-functions-rs/tree/master/azure-functions-codegen) - The `azure-functions-codegen` crate that defines the procedural macros that are used when writing Azure Functions with Rust.
* [azure-functions-sdk](https://github.com/peterhuene/azure-functions-rs/tree/master/azure-functions-sdk) - The `azure-functions-sdk` crate that implements the `cargo func` command.
* [azure-functions-shared](https://github.com/peterhuene/azure-functions-rs/tree/master/azure-functions-shared) - The `azure-functions-shared` crate that defines types and functions that are shared between the `azure-functions-codegen` and `azure-functions` crates.
    * Note: the `azure-functions-shared/protobuf` directory is the git submodule for [Azure Functions Language Worker Protocol](https://github.com/Azure/azure-functions-language-worker-protobuf).
* [azure-functions-shared-codegen](https://github.com/peterhuene/azure-functions-rs/tree/master/azure-functions-shared-codegen) - The `azure-functions-shared-codegen` crate that defines the procedural macros used by the shared `azure-functions-shared` crate.
* [examples](https://github.com/peterhuene/azure-functions-rs/tree/master/examples) - The directory containing example Azure Functions.

## Building

Build at the root of the repository to build both the `azure-functions-codegen` and the `azure-functions` libraries using `cargo build`:

```
$ cargo build
```

## Running tests

Use `cargo test` to run the tests:

```
$ cargo test
```
