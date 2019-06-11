# Azure Functions for Rust
<link rel="stylesheet" href="https://use.fontawesome.com/releases/v5.8.1/css/all.css" integrity="sha384-50oBUHEmvpQ+1lW4y57PTFmhCaXp0ML5d60M1M7uH2+nqUivzIebhndOJK28anvf" crossorigin="anonymous">

[![crates.io](https://img.shields.io/crates/v/azure-functions.svg)](https://crates.io/crates/azure-functions)
[![docs.rs](https://docs.rs/azure-functions/badge.svg)](https://docs.rs/azure-functions)
[![All Contributors](https://img.shields.io/badge/all_contributors-1-orange.svg?style=flat-square)](#contributors)
[![Gitter](https://badges.gitter.im/azure-functions-rs/community.svg)](https://gitter.im/azure-functions-rs/community?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge)
[![Build Status](https://dev.azure.com/azure-functions-rs/Azure%20Functions%20for%20Rust/_apis/build/status/peterhuene.azure-functions-rs?branchName=master)](https://dev.azure.com/azure-functions-rs/Azure%20Functions%20for%20Rust/_build/latest?definitionId=2&branchName=master)
[![Dependabot Status](https://api.dependabot.com/badges/status?host=github&repo=peterhuene/azure-functions-rs)](https://dependabot.com)
[![license](https://img.shields.io/crates/l/azure-functions.svg)](https://github.com/peterhuene/azure-functions-rs/blob/master/LICENSE)

A framework for implementing [Azure Functions](https://azure.microsoft.com/en-us/services/functions/)
in [Rust](https://www.rust-lang.org/).

> :triangular_flag_on_post: **Disclaimer**  
> Although the maintainer of this project is a Microsoft employee, this project is not an officially recognized Microsoft product and is not an endorsement of any future product offering from Microsoft.  
> _Microsoft and Azure are registered trademarks of Microsoft Corporation._

*If you would like the Azure Functions team to consider supporting Rust, please [vote up this feedback item](https://feedback.azure.com/forums/355860-azure-functions/suggestions/36818512-support-for-native-rust-azure-functions).*

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

## Get Started

- [More Examples](https://github.com/peterhuene/azure-functions-rs/tree/master/examples)
- [Documentation](https://docs.rs/azure-functions/0.9.0/azure_functions/)
- [Installation](#installation)
- [Contributing](https://github.com/peterhuene/azure-functions-rs/blob/master/CONTRIBUTING.md)

## Table of Contents

- [Installation](#installation)
  - [Requirements](#requirements)
    - [.Net Core SDK](#.net-core-sdk)
    - [Azure Functions Core Tools](#azure-functions-core-tools)
  - [Installing the Azure Functions for Rust SDK](#installing-the-azure-functions-for-rust-sdk)

- [Creating a new Azure Functions application](#creating-a-new-azure-functions-application)
- [Adding a simple HTTP-triggered application](#adding-a-simple-http-triggered-application)
- [Building the Azure Functions application](#building-the-azure-functions-application)
- [Running the Azure Functions application](#running-the-azure-functions-application)
- [Debugging the Azure Functions application](#debugging-the-azure-functions-application)
- [Deploying the Azure Functions application](#deploying-the-azure-functions-application)
- [Azure Functions Bindings](#azure-functions-bindings)
  - [Bindings in Rust](#bindings-in-rust)
    - [Input bindings](#input-bindings)
    - [Input-output (inout) bindings](#input-output-inout-bindings)
    - [Output bindings](#output-bindings)
- [Contributors](#contributors)

## Installation

### Requirements

#### .NET Core SDK

A .NET Core SDK is required to synchronize Azure Functions Host binding extensions.

For example, using the Cosmos DB bindings will need the `Microsoft.Azure.WebJobs.Extensions.CosmosDB` extensions installed for the Azure Functions Host.

This happens automatically by Azure Functions for Rust when the application is initialized.

Follow the [download instructions for the 2.2 .NET Core SDK](https://dotnet.microsoft.com/download/dotnet-core/2.2) to install for Windows, macOS, or your Linux distro.

#### Azure Functions Core Tools

Install version 2 or higher of the [Azure Functions Core Tools](https://docs.microsoft.com/en-us/azure/azure-functions/functions-run-local#install-the-azure-functions-core-tools).

If you are on Windows, you must add `%ProgramFiles%\nodejs\node_modules\azure-functions-core-tools\bin` (where `func.exe` is located) to the `PATH` environment variable.

### Installing the Azure Functions for Rust SDK

Install the Azure Functions for Rust SDK using `cargo install`:

```bash
cargo install azure-functions-sdk
```

This installs a new cargo command named `func` that can be used to create and run new Azure Functions applications.

## Creating a new Azure Functions application

Use the `cargo func new-app` command to create a new Azure Functions application:

``` bash
cargo func new-app hello
```

This will create a new application in the `./hello` directory with a module named `functions` where the exported Azure Functions are expected to be placed.

## Adding a simple HTTP-triggered application

Use the `cargo func new` command to create a new HTTP-triggered Azure Function named `hello`:

``` bash
cargo func new http -n hello
```

The source for the function will be in `src/functions/hello.rs`.

## Building the Azure Functions application

To build your Azure Functions application, just use `cargo build`:

``` bash
cargo build
```

If you are using a nightly compiler, you can enable improved error messages during compilation.

Add the following to your `Cargo.toml`:

``` toml
[features]
unstable = ["azure-functions/unstable"]
```

Build your application:

``` bash
cargo build --features unstable
```

This enables Azure Functions for Rust to emit diagnostic messages that will include the position of an error within an attribute.

## Running the Azure Functions application

To build and run your Azure Functions application, use `cargo func run`:

``` bash
cargo func run
```

The `cargo func run` command builds and runs your application locally using the Azure Function Host that was
installed by the Azure Functions Core Tools.

By default, the host will be configured to listen on `0.0.0.0:8080`.

For the `hello` function added previously, it can be invoked from `http://localhost:8080/api/hello`.

## Debugging the Azure Functions application

The easiest way to debug the Azure Functions application is to use [Visual Studio Code](https://code.visualstudio.com/) with the [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb) extension.

By default, the Azure Functions for Rust SDK will create a Visual Studio Code launch configuration when you run `cargo func new-app`.

This will enable a `Debug` launch configuration that will build and run your application locally before attaching the `lldb` debugger to the Rust worker process.

## Deploying the Azure Functions application

In the future, there will be a `cargo func deploy` command to deploy the Azure Functions application directly to Azure.

Until that time, you must manually build and push the Docker image to a repository that can be accessed by Azure.

> Note: this requires [Docker](https://www.docker.com/get-started) that is at least *18.06* for the experimental BuildKit support.

**To enable the BuildKit support, set the `DOCKER_BUILDKIT` environment variable to `1` before running `docker build`.**

Start by building your image with `docker build -t $TAG_NAME .`:

``` bash
docker build -t $TAG_NAME .
```

Where `$TAG_NAME` is the tag name to use (e.g. `username/my-functions-app`).

Use `docker push` to push the image to a repository that is accessible to Azure Functions.

``` bash
docker push $TAG_NAME
```

Create the Function App in [Azure](https://portal.azure.com) using the "Linux (Preview)" OS.  Under the "Publish" setting, select "Docker Image".

From the "Configure Container" section, select the repository and enter the image you pushed.

That's it! Once the Functions App starts in Azure, you should be able to view the functions and run them.

## Azure Functions Bindings

Azure Functions supports a [wide variety of input and output bindings](https://docs.microsoft.com/en-us/azure/azure-functions/functions-triggers-bindings) that can be used by a function.

In a language like C#, parameters can be annotated with attributes describing how the parameters are bound.

Rust does not support attributes on parameters, so the `#[binding]` attribute is applied on the function with a name that matches the parameter's identifier.  The arguments to the attribute depend on the binding type.

The `#[func]` attribute is used to turn an ordinary Rust function into an Azure Function.  It works by examining the parameters and return type to the function and automatically mapping them to corresponding bindings.

The current list of supported bindings:

| Rust Type                                                                                                                  | Azure Functions Binding             | Direction      | Vec\<T> |
|----------------------------------------------------------------------------------------------------------------------------|-------------------------------------|----------------|---------|
| [Blob](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.Blob.html)                                   | Input and Ouput Blob                | in, inout, out | No      |
| [BlobTrigger](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.BlobTrigger.html)                     | Blob Trigger                        | in, inout      | No      |
| [CosmosDbDocument](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.CosmosDbDocument.html)           | Input and Output Cosmos DB Document | in, out        | Yes     |
| [CosmosDbTrigger](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.CosmosDbTrigger.html)             | Cosmos DB Trigger                   | in             | No      |
| [EventGridEvent](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.EventGridEvent.html)               | Event Grid Trigger                  | in             | No      |
| [EventHubMessage](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.EventHubMessage.html)             | Event Hub Output Message            | out            | Yes     |
| [EventHubTrigger](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.EventHubTrigger.html)             | Event Hub Trigger                   | in             | No      |
| [GenericInput](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.GenericInput.html)                   | Generic Input                       | in             | No      |
| [GenericOutput](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.GenericOutput.html)                 | Generic Output                      | out            | No      |
| [GenericTrigger](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.GenericTrigger.html)               | Generic Trigger                     | in             | No      |
| [HttpRequest](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.HttpRequest.html)                     | HTTP Trigger                        | in             | No      |
| [HttpResponse](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.HttpResponse.html)                   | Output HTTP Response                | out            | No      |
| [QueueMessage](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.QueueMessage.html)                   | Output Queue Message                | out            | Yes     |
| [QueueTrigger](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.QueueTrigger.html)                   | Queue Trigger                       | in             | No      |
| [SendGridMessage](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.SendGridMessage.html)             | SendGrid Email Message              | out            | Yes     |
| [ServiceBusMessage](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.ServiceBusMessage.html)         | Service Bus Output Message          | out            | Yes     |
| [ServiceBusTrigger](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.ServiceBusTrigger.html)         | Service Bus Trigger                 | in             | No      |
| [SignalRConnectionInfo](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.SignalRConnectionInfo.html) | SignalR Connection Info             | in             | No      |
| [SignalRGroupAction](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.SignalRGroupAction.html)       | SignalR Group Action                | out            | Yes     |
| [SignalRMessage](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.SignalRMessage.html)               | SignalR Message                     | out            | Yes     |
| [Table](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.Table.html)                                 | Input and Ouput Table               | in, out        | No      |
| [TimerInfo](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.TimerInfo.html)                         | Timer Trigger                       | in             | No      |
| [TwilioSmsMessage](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.TwilioSmsMessage.html)           | Twilio SMS Message Output | out     | Yes            | Yes     |
| [Context](https://docs.rs/azure-functions/latest/azure_functions/struct.Context.html)*                                     | Invocation Context                  | N/A            | N/A     |

\****Note: the `Context` binding is not an Azure Functions binding; it is used to pass information about the function being invoked.***

More bindings will be implemented in the future, including support for retreiving data from custom bindings.

### Bindings in Rust

Azure Functions for Rust automatically infers the direction of bindings depending on how the binding is used in a function's declaration.

#### Input bindings

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

#### Input-output (inout) bindings

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

#### Output bindings

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
* `SendGridMessage`
* `ServiceBusMessage`
* `SignalRGroupAction`
* `SignalRMessage`
* `TwilioSmsMessage`

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

## Contributors

Thanks goes to these wonderful people ([emoji key](https://allcontributors.org/docs/en/emoji-key)):

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore -->
<table><tr><td align="center"><a href="https://github.com/peterhuene"><img src="https://avatars3.githubusercontent.com/u/509666?v=4" width="100px;" alt="Peter Huene"/><br /><sub><b>Peter Huene</b></sub></a><br /><a href="https://github.com/peterhuene/azure-functions-rs/commits?author=peterhuene" title="Code">💻</a> <a href="https://github.com/peterhuene/azure-functions-rs/issues?q=author%3Apeterhuene" title="Bug reports">🐛</a> <a href="https://github.com/peterhuene/azure-functions-rs/commits?author=peterhuene" title="Documentation">📖</a> <a href="#ideas-peterhuene" title="Ideas, Planning, & Feedback">🤔</a> <a href="#infra-peterhuene" title="Infrastructure (Hosting, Build-Tools, etc)">🚇</a> <a href="#maintenance-peterhuene" title="Maintenance">🚧</a> <a href="#platform-peterhuene" title="Packaging/porting to new platform">📦</a> <a href="#review-peterhuene" title="Reviewed Pull Requests">👀</a> <a href="https://github.com/peterhuene/azure-functions-rs/commits?author=peterhuene" title="Tests">⚠️</a> <a href="#tutorial-peterhuene" title="Tutorials">✅</a></td></tr></table>

<!-- ALL-CONTRIBUTORS-LIST:END -->

This project follows the [all-contributors](https://github.com/all-contributors/all-contributors) specification. Contributions of any kind welcome!