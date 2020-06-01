# Notice

:warning: 

This project is no longer being actively maintained by the original author.

Forks or derivative works are encouraged!

# Azure Functions for Rust
<link rel="stylesheet" href="https://use.fontawesome.com/releases/v5.8.1/css/all.css" integrity="sha384-50oBUHEmvpQ+1lW4y57PTFmhCaXp0ML5d60M1M7uH2+nqUivzIebhndOJK28anvf" crossorigin="anonymous">

[crates-status]: https://img.shields.io/crates/v/azure-functions.svg
[crates-url]: https://crates.io/crates/azure-functions
[docs-status]: https://docs.rs/azure-functions/badge.svg
[docs-url]: https://docs.rs/azure-functions
[all-contributors-status]: https://img.shields.io/badge/all_contributors-7-orange.svg?style=flat-square
[gitter-status]: https://badges.gitter.im/azure-functions-rs/community.svg
[gitter-url]: https://gitter.im/azure-functions-rs/community?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge
[build-status]: https://github.com/peterhuene/azure-functions-rs/workflows/CI/badge.svg?branch=master
[build-url]: https://github.com/peterhuene/azure-functions-rs/actions?query=workflow%3ACI%20branch%3Amaster
[dependabot-status]: https://api.dependabot.com/badges/status?host=github&repo=peterhuene/azure-functions-rs
[dependabot-url]: https://dependabot.com
[license-status]: https://img.shields.io/crates/l/azure-functions.svg
[license-url]: https://github.com/peterhuene/azure-functions-rs/blob/master/LICENSE

[![crates-status]][crates-url]
[![docs-status]][docs-url]
[![all-contributors-status]](#contributors)
[![gitter-status]][gitter-url]
[![build-status]][build-url]
[![dependabot-status]][dependabot-url]
[![license-status]][license-url]

A framework for implementing [Azure Functions](https://azure.microsoft.com/en-us/services/functions/)
in [Rust](https://www.rust-lang.org/).

> :triangular_flag_on_post: **Disclaimer**  
> This project is not an officially recognized Microsoft product and is not an endorsement of any future product offering from Microsoft.
>
> _Microsoft and Azure are registered trademarks of Microsoft Corporation._

*If you would like the Azure Functions team to consider supporting Rust, please [vote up this feedback item](https://feedback.azure.com/forums/355860-azure-functions/suggestions/36818512-support-for-native-rust-azure-functions).*

## Example

A simple HTTP-triggered Azure Function:

```rust
use azure_functions::{
    bindings::{HttpRequest, HttpResponse},
    func,
};

#[func]
pub fn greet(req: HttpRequest) -> HttpResponse {
    format!(
        "Hello from Rust, {}!\n",
        req.query_params().get("name").map_or("stranger", |x| x)
    )
    .into()
}
```

Azure Functions for Rust also supports [async](https://rust-lang.github.io/async-book/01_getting_started/04_async_await_primer.html) functions:

```rust
use azure_functions::{
    bindings::{HttpRequest, HttpResponse},
    func,
};
use futures::future::ready;

#[func]
pub async fn greet_async(req: HttpRequest) -> HttpResponse {
    // Use ready().await to simply demonstrate the async/await feature
    ready(format!(
        "Hello from Rust, {}!\n",
        req.query_params().get("name").map_or("stranger", |x| x)
    ))
    .await
    .into()
}
```

See [Building an async Azure Functions application](#building-an-async-azure-functions-application) for more information.

## Get Started

- [More Examples](https://github.com/peterhuene/azure-functions-rs/tree/master/examples)
- [Documentation](https://docs.rs/azure-functions/0.11.0/azure_functions/)
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
- [Building an async Azure Functions application](#building-an-async-azure-functions-application)
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

## Building an async Azure Functions application

To build with support for async Azure Functions, add the following to your `Cargo.toml`:

```toml
[dependencies]
futures-preview = "0.3.0-alpha.19"
```

And then build:

```bash
cargo build
```

## Running the Azure Functions application

To build and run your Azure Functions application, use `cargo func run`:

``` bash
cargo func run
```

If you need to enable the `unstable` feature, pass the `--features` option to cargo:

```bash
cargo func run -- --features unstable
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

Create the "Function App (Classic)" in [Azure](https://portal.azure.com) using the "Linux (Preview)" OS.  Under the "Publish" setting, select "Docker Image".

From the "Configure Container" section, select the repository and enter the image you pushed.

That's it! Once the Functions App starts in Azure, you should be able to view the functions and run them.

## Azure Functions Bindings

Azure Functions supports a [wide variety of input and output bindings](https://docs.microsoft.com/en-us/azure/azure-functions/functions-triggers-bindings) that can be used by a function.

In a language like C#, parameters can be annotated with attributes describing how the parameters are bound.

Rust does not support attributes on parameters, so the `#[binding]` attribute is applied on the function with a name that matches the parameter's identifier.  The arguments to the attribute depend on the binding type.

The `#[func]` attribute is used to turn an ordinary Rust function into an Azure Function.  It works by examining the parameters and return type to the function and automatically mapping them to corresponding bindings.

The current list of supported bindings:

| Rust Type                                                                                                                              | Azure Functions Binding             | Direction      | Vec\<T> |
|----------------------------------------------------------------------------------------------------------------------------------------|-------------------------------------|----------------|---------|
| [Blob](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.Blob.html)                                               | Input and Ouput Blob                | in, inout, out | No      |
| [BlobTrigger](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.BlobTrigger.html)                                 | Blob Trigger                        | in, inout      | No      |
| [CosmosDbDocument](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.CosmosDbDocument.html)                       | Input and Output Cosmos DB Document | in, out        | Yes     |
| [CosmosDbTrigger](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.CosmosDbTrigger.html)                         | Cosmos DB Trigger                   | in             | No      |
| [DurableActivityContext](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.DurableActivityContext.html)           | Durable Activity Trigger            | in             | No      |
| [DurableOrchestrationClient](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.DurableOrchestrationClient.html)   | Durable Orchestration Client        | in             | No      |
| [DurableOrchestrationContext](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.DurableOrchestrationContext.html) | Durable Orchestration Trigger       | in             | No      |
| [EventGridEvent](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.EventGridEvent.html)                           | Event Grid Trigger                  | in             | No      |
| [EventHubMessage](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.EventHubMessage.html)                         | Event Hub Output Message            | out            | Yes     |
| [EventHubTrigger](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.EventHubTrigger.html)                         | Event Hub Trigger                   | in             | No      |
| [GenericInput](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.GenericInput.html)                               | Generic Input                       | in             | No      |
| [GenericOutput](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.GenericOutput.html)                             | Generic Output                      | out            | No      |
| [GenericTrigger](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.GenericTrigger.html)                           | Generic Trigger                     | in             | No      |
| [HttpRequest](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.HttpRequest.html)                                 | HTTP Trigger                        | in             | No      |
| [HttpResponse](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.HttpResponse.html)                               | Output HTTP Response                | out            | No      |
| [QueueMessage](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.QueueMessage.html)                               | Output Queue Message                | out            | Yes     |
| [QueueTrigger](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.QueueTrigger.html)                               | Queue Trigger                       | in             | No      |
| [SendGridMessage](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.SendGridMessage.html)                         | SendGrid Email Message              | out            | Yes     |
| [ServiceBusMessage](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.ServiceBusMessage.html)                     | Service Bus Output Message          | out            | Yes     |
| [ServiceBusTrigger](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.ServiceBusTrigger.html)                     | Service Bus Trigger                 | in             | No      |
| [SignalRConnectionInfo](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.SignalRConnectionInfo.html)             | SignalR Connection Info             | in             | No      |
| [SignalRGroupAction](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.SignalRGroupAction.html)                   | SignalR Group Action                | out            | Yes     |
| [SignalRMessage](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.SignalRMessage.html)                           | SignalR Message                     | out            | Yes     |
| [Table](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.Table.html)                                             | Input and Ouput Table               | in, out        | No      |
| [TimerInfo](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.TimerInfo.html)                                     | Timer Trigger                       | in             | No      |
| [TwilioSmsMessage](https://docs.rs/azure-functions/latest/azure_functions/bindings/struct.TwilioSmsMessage.html)                       | Twilio SMS Message Output | out     | Yes            | Yes     |

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
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tr>
    <td align="center"><a href="https://github.com/peterhuene"><img src="https://avatars3.githubusercontent.com/u/509666?v=4" width="100px;" alt=""/><br /><sub><b>Peter Huene</b></sub></a><br /><a href="https://github.com/peterhuene/azure-functions-rs/commits?author=peterhuene" title="Code">💻</a> <a href="https://github.com/peterhuene/azure-functions-rs/issues?q=author%3Apeterhuene" title="Bug reports">🐛</a> <a href="https://github.com/peterhuene/azure-functions-rs/commits?author=peterhuene" title="Documentation">📖</a> <a href="#ideas-peterhuene" title="Ideas, Planning, & Feedback">🤔</a> <a href="#infra-peterhuene" title="Infrastructure (Hosting, Build-Tools, etc)">🚇</a> <a href="#maintenance-peterhuene" title="Maintenance">🚧</a> <a href="#platform-peterhuene" title="Packaging/porting to new platform">📦</a> <a href="https://github.com/peterhuene/azure-functions-rs/pulls?q=is%3Apr+reviewed-by%3Apeterhuene" title="Reviewed Pull Requests">👀</a> <a href="https://github.com/peterhuene/azure-functions-rs/commits?author=peterhuene" title="Tests">⚠️</a> <a href="#tutorial-peterhuene" title="Tutorials">✅</a></td>
    <td align="center"><a href="https://github.com/rylev"><img src="https://avatars3.githubusercontent.com/u/1327285?v=4" width="100px;" alt=""/><br /><sub><b>Ryan Levick</b></sub></a><br /><a href="https://github.com/peterhuene/azure-functions-rs/commits?author=rylev" title="Code">💻</a> <a href="#ideas-rylev" title="Ideas, Planning, & Feedback">🤔</a> <a href="#maintenance-rylev" title="Maintenance">🚧</a> <a href="https://github.com/peterhuene/azure-functions-rs/pulls?q=is%3Apr+reviewed-by%3Arylev" title="Reviewed Pull Requests">👀</a> <a href="#infra-rylev" title="Infrastructure (Hosting, Build-Tools, etc)">🚇</a></td>
    <td align="center"><a href="https://thomaseckert.org"><img src="https://avatars3.githubusercontent.com/u/29112081?v=4" width="100px;" alt=""/><br /><sub><b>Thomas Eckert</b></sub></a><br /><a href="https://github.com/peterhuene/azure-functions-rs/commits?author=t-eckert" title="Code">💻</a> <a href="#content-t-eckert" title="Content">🖋</a> <a href="#design-t-eckert" title="Design">🎨</a> <a href="https://github.com/peterhuene/azure-functions-rs/commits?author=t-eckert" title="Documentation">📖</a> <a href="#ideas-t-eckert" title="Ideas, Planning, & Feedback">🤔</a> <a href="#infra-t-eckert" title="Infrastructure (Hosting, Build-Tools, etc)">🚇</a> <a href="#maintenance-t-eckert" title="Maintenance">🚧</a> <a href="https://github.com/peterhuene/azure-functions-rs/pulls?q=is%3Apr+reviewed-by%3At-eckert" title="Reviewed Pull Requests">👀</a> <a href="#tutorial-t-eckert" title="Tutorials">✅</a></td>
    <td align="center"><a href="https://brokenco.de/"><img src="https://avatars0.githubusercontent.com/u/26594?v=4" width="100px;" alt=""/><br /><sub><b>R. Tyler Croy</b></sub></a><br /><a href="https://github.com/peterhuene/azure-functions-rs/commits?author=rtyler" title="Code">💻</a> <a href="https://github.com/peterhuene/azure-functions-rs/commits?author=rtyler" title="Documentation">📖</a> <a href="https://github.com/peterhuene/azure-functions-rs/pulls?q=is%3Apr+reviewed-by%3Artyler" title="Reviewed Pull Requests">👀</a> <a href="#ideas-rtyler" title="Ideas, Planning, & Feedback">🤔</a></td>
    <td align="center"><a href="https://github.com/dmolokanov"><img src="https://avatars2.githubusercontent.com/u/6630003?v=4" width="100px;" alt=""/><br /><sub><b>Denis Molokanov</b></sub></a><br /><a href="https://github.com/peterhuene/azure-functions-rs/commits?author=dmolokanov" title="Code">💻</a> <a href="https://github.com/peterhuene/azure-functions-rs/commits?author=dmolokanov" title="Tests">⚠️</a> <a href="#design-dmolokanov" title="Design">🎨</a> <a href="#ideas-dmolokanov" title="Ideas, Planning, & Feedback">🤔</a></td>
    <td align="center"><a href="https://github.com/slyons"><img src="https://avatars3.githubusercontent.com/u/41403?v=4" width="100px;" alt=""/><br /><sub><b>Scott Lyons</b></sub></a><br /><a href="https://github.com/peterhuene/azure-functions-rs/commits?author=slyons" title="Code">💻</a> <a href="https://github.com/peterhuene/azure-functions-rs/commits?author=slyons" title="Tests">⚠️</a> <a href="#design-slyons" title="Design">🎨</a> <a href="#ideas-slyons" title="Ideas, Planning, & Feedback">🤔</a></td>
    <td align="center"><a href="https://github.com/ajnirp"><img src="https://avatars1.githubusercontent.com/u/1688456?v=4" width="100px;" alt=""/><br /><sub><b>Rohan Prinja</b></sub></a><br /><a href="https://github.com/peterhuene/azure-functions-rs/commits?author=ajnirp" title="Code">💻</a></td>
  </tr>
  <tr>
    <td align="center"><a href="https://www.cbrevik.com"><img src="https://avatars2.githubusercontent.com/u/4932625?v=4" width="100px;" alt=""/><br /><sub><b>Christian Brevik</b></sub></a><br /><a href="https://github.com/peterhuene/azure-functions-rs/issues?q=author%3Acbrevik" title="Bug reports">🐛</a> <a href="https://github.com/peterhuene/azure-functions-rs/commits?author=cbrevik" title="Code">💻</a> <a href="#tutorial-cbrevik" title="Tutorials">✅</a></td>
    <td align="center"><a href="https://github.com/dbcfd"><img src="https://avatars2.githubusercontent.com/u/1475860?v=4" width="100px;" alt=""/><br /><sub><b>Danny Browning</b></sub></a><br /><a href="https://github.com/peterhuene/azure-functions-rs/commits?author=dbcfd" title="Code">💻</a> <a href="#maintenance-dbcfd" title="Maintenance">🚧</a></td>
  </tr>
</table>

<!-- markdownlint-enable -->
<!-- prettier-ignore-end -->
<!-- ALL-CONTRIBUTORS-LIST:END -->

This project follows the [all-contributors](https://github.com/all-contributors/all-contributors) specification. Contributions of any kind welcome!
