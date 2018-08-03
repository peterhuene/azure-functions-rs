# Example Timer Azure Function

This project is an example of a simple timer-triggered Azure Function.

## Example function implementation

The example timer-triggered Azure Function that runs every minute:

```rust
use azure_functions::bindings::TimerInfo;
use azure_functions::func;

#[func]
#[binding(name = "info", schedule = "0 */1 * * * *")]
pub fn timer(info: &TimerInfo) {
    info!("Hello from Rust!");
    info!("Timer information: {:?}", info);
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
cd azure-functions-rs/examples/timer
cargo run --release -- init --worker-path /tmp/timer-example/rust_worker --script-root /tmp/timer-example/root
```

## Start the Azure Functions Host

Run the following commands to start the Azure Functions Host:

```
cd azure-functions-host/src/WebJobs.Script.WebHost
PATH=/tmp/timer-example:$PATH AzureWebJobsScriptRoot=/tmp/timer-example/root AzureWebJobsStorage=$CONNECTION_STRING dotnet run
```

Where `$CONNECTION_STRING` is the Azure Storage connection string the Azure Functions host should use.

_Note: the syntax above works on macOS and Linux; on Windows, set the environment variables before running `dotnet run`._

## Invoke the `timer` function

The example function is automatically invoked by the Azure Functions Host when the timer expires.

Wait a minute and then check the Azure Functions Host output.

With any luck, you should see the following output from the Azure Functions Host:

```
info: Function.timer[0]
      => System.Collections.Generic.Dictionary`2[System.String,System.Object]
      Executing 'Functions.timer' (Reason='Timer fired at 2018-07-15T23:34:00.0200030-07:00', Id=99936a5b-8df1-48c7-97b7-afec3b579215)
info: Worker.Rust.7ed9c518-6f7e-4c0b-bc5d-0fdb77a0d1b1[0]
      Hello every minute from Rust!
info: Function.timer[0]
      => System.Collections.Generic.Dictionary`2[System.String,System.Object]
      Executed 'Functions.timer' (Succeeded, Id=99936a5b-8df1-48c7-97b7-afec3b579215)
```
