# Azure Functions for Rust

[![crates.io](https://img.shields.io/crates/v/azure-functions.svg)](https://crates.io/crates/azure-functions)
[![docs.rs](https://docs.rs/azure-functions/badge.svg)](https://docs.rs/azure-functions)
[![CircleCI branch](https://img.shields.io/circleci/project/github/peterhuene/azure-functions-rs/master.svg)](https://circleci.com/gh/peterhuene/azure-functions-rs/tree/master)
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
#[binding(name = "req", auth_level = "anonymous")]
pub fn greet(req: &HttpRequest) -> HttpResponse {
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

## Installing the Azure Functions for Rust SDK

Install the Azure Functions for Rust SDK using `cargo install`:


```bash
$ cargo install azure-functions-sdk
```

This installs a new cargo command named `func` that can be used to create new Azure Functions
applications, build, and easily run them locally inside of a Docker container.

## Creating a new Azure Functions application

Use the `cargo func new-app` command to create a new Azure Functions application:

```bash
$ cargo func new-app hello
```

This will create a new application in the `./hello` directory with a module named `functions` where the exported Azure Functions are expected to be placed.

Inside of `src/functions/mod.rs` is a declaration of all exported functions.  A function **will not be loaded by the Azure Functions Host** if it is not declared in the list of exported functions.

## Building the Azure Functions application

To build your Azure Functions application, use `cargo func build`:

```
$ cargo func build
```

**Note: this requires [Docker](https://www.docker.com/get-started) that is at least *18.06* for the experimental BuildKit support.**

The `cargo func build` command is responsible for building a Docker image that can be used to run the Azure Functions application locally.

It will download a Docker image that contains a recent nightly Rust toolset and any dependencies that
Azure Functions for Rust needs to build and then builds the application inside an intermediary image
where the `target/` directory is cached to enable incremental builds.

**Note: the very first build will take a long time to download the base build image and then compile Azure Functions for Rust with its dependencies; after the first build, the built dependencies will be cached and thus it should build much faster.**

## Running the Azure Functions application

To build and run your Azure Functions application, use `cargo func run`:

```
$ cargo func run
```

The `cargo func run` command is responsible for building a Docker image and then running it inside a Docker container.

By default, it exposes port `8080` as the port for the Azure Functions Host running inside the container.

After the Azure Functions application starts, `http://localhost:8080` should load the welcome page for
an Azure Functions application.  The HTTP Azure Functions can then be triggered, by default, with
`http://localhost:8080/api/$NAME` (where `$NAME` is the name of the exported Azure Function).

## Deploying the Azure Functions application

In the future, there will be a `cargo func deploy` command to deploy the Azure Functions application directly to Azure.

Until that time, you must manually push the Docker image to a repository that can be accessed by Azure.

Start by building your image with `cargo func build`:

```
$ cargo func build
```

This creates a tag named `azure-functions/<name>` where `<name>` is the name of your Rust crate.

While this is a useful tag for running the Azure Functions application locally, it's not useful for pushing the image to a remote repository.

Create a new tag that is for the proper user for your repository:

```
$ docker tag azure-functions/<name> <user>/<name>
```

Use `docker push` to push the image that was previously built with `cargo func build`:

```
$ docker push <new-tag-name>
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

| Rust Type                                 | Azure Functions Binding | Direction      |
|-------------------------------------------|-------------------------|----------------|
| `azure_functions::bindings::Blob`         | Input and Ouput Blob    | in, inout, out |
| `azure_functions::bindings::BlobTrigger`  | Blob Trigger            | in, inout      |
| `azure_functions::bindings::HttpRequest`  | HTTP Trigger            | in             |
| `azure_functions::bindings::HttpResponse` | Output HTTP Response    | out            |
| `azure_functions::bindings::QueueTrigger` | Queue Trigger           | in             |
| `azure_functions::bindings::QueueMessage` | Output Queue Message    | out            |
| `azure_functions::bindings::Table`        | Input and Ouput Table   | in, out        |
| `azure_functions::bindings::TimerInfo`    | Timer Trigger           | in             |
| `azure_functions::Context`*               | Invocation Context      | n/a            |

\****Note: the `Context` binding is not an Azure Functions binding; it is used to pass information about the function being invoked.***

More bindings will be implemented in the future, including support for retreiving data from custom bindings.

## Bindings in Rust

Azure Functions for Rust automatically infers the direction of bindings depending on how the binding is used in a function's declaration:

* Parameters passed by immutable reference `&T`, where `T` is a trigger or input binding type, are inferred to be bindings with an `in` direction.

  ```rust
  #[func]
  ...
  pub fn example(..., blob: &Blob) {
      ...
  }
  ```

* Parameters passed by mutable reference `&mut T`, where `T` is a trigger or input binding type that supports the `inout` direction, are inferred to be bindings with an `inout` direction.
**Note: `inout` direction bindings are currently not implemented for languages other than C#.  See [this issue](https://github.com/Azure/azure-functions-host/issues/49) regarding this problem with the Azure Functions Host.**

  ```rust
  #[func]
  ...
  pub fn example(..., blob: &mut Blob) {
      ...
  }
  ```

* Functions that return a type `T`, where `T` is an output binding type, or a tuple of output binding types, are inferred to be bindings with an `out` direction.  Functions may also return `Option<T>` for any output binding type `T`; a `None` value will skip outputting a value.

  ```rust
  #[func]
  ...
  pub fn example(...) -> Blob {
      ...
  }
  ```

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

  For functions that return a single output binding type, the binding has a special name of `$return`
  and is treated as the "return value" of the function.

  For functions that return a tuple of output binding types, the first value of the tuple has the binding name
  of `$return` and is treated as the "return value" of the function.  The remaining values have binding names `output1`, `output2`, ..., `output(N-1)` where `N` is the number of types in the tuple, and are
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

## Installing development dependencies

An OpenSSL library is required to build the `grpcio` dependency of Azure Functions for Rust.

### Installing OpenSSL on macOS

Use Homebrew to install the `openssl` package:

```
$ brew install openssl
$ export OPENSSL_ROOT_DIR=$(brew --prefix openssl)
```

Note that the `OPENSSL_ROOT_DIR` variable is only required when having to build the dependencies of Azure Functions for Rust.

### Installing OpenSSL on Ubuntu / Debian

Use `apt-get` to install the OpenSSL development package:

```
$ sudo apt-get install libssl-dev
```

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

## Running with a native Azure Functions Host

The `cargo func run` command builds and runs the Azure Functions application with Docker and runs the Azure Functions Host in a Linux container.

It is possible to instead run the Rust application natively using a locally running Azure Functions Host.

### Clone the Azure Functions Host repository

Clone the [Azure Functions Host](https://github.com/Azure/azure-functions-host) repository to a local directory:

```
$ git clone git@github.com:Azure/azure-functions-host.git
```

### Install a .NET Core SDK

The Azure Functions Host is implemented with .NET Core.

Install the latest [.NET Core SDK](https://dotnet.microsoft.com/download) so you can build and run the Azure Functions Host.

### Build the Azure Functions for Rust application

Build your test Azure Functions for Rust application using `cargo run` instead of `cargo func build`:

```
$ cargo run --release -- init --script-root ./root --worker-path ./root/rust_worker --sync
```

This command will create your Azure Functions App in `./root`.  The Azure Functions Host will be
configured to use this location for the application later.

### Configure the Azure Funtions Host

Start by building the Azure Functions Host:

```
$ cd azure-functions-host/src/WebJobs.Script.WebHost
$ dotnet build
```

For the Azure Functions Host to support Rust, we need to copy the worker configuration file to the appropriate location:

```
$ mkdir azure-functions-host/src/WebJobs.Script.WebHost/bin/Debug/netcoreapp2.1/workers/rust
$ cp azure-functions-rs/azure-functions/worker.config.json azure-functions-host/src/WebJobs.Script.WebHost/bin/Debug/netcoreapp2.1/workers/rust/
```

If you want to change the default logging level of the host, add the following to `azure-functions-host/src/WebJobs.Script.WebHost/bin/Debug/netcoreapp2.1/appsettings.json`:

```json
{
    "AzureFunctionsJobHost": {
        "Logging": {
            "Console": {
                "IsEnabled": true
            },
            "LogLevel": {
                "Default": "Information"
            }
        }
    }
}
```

### Start the Azure Functions Host

There are three thigns the Azure Funcions Host needs to know to run the Rust application:

* The path to the "script root" (e.g. `./root` above in the "Building the Azure Functions for Rust application" section).
* The location of the Rust worker (built by `cargo run` above).
* The default connection string to use for Azure Storage services (needed for most bindings).  This can be found under the "Access Keys" settings for your Azure Storage account.


Run the Azure Functions Host with the necessary environment:

```
$ export AzureWebJobsScriptRoot=<full_path_to_script_root>
$ export AzureWebJobsStorage=<storage_connection_string>
$ export PATH=$PATH:$AzureWebJobsScriptRoot
$ cd azure-functions-host/src/WebJobs.Script.WebHost
$ dotnet run
```

The Azure Functions Host should start and can now access your Azure Functions application via `http://localhost:5000`.
