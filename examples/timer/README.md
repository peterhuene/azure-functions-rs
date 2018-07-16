# Example Timer Azure Function

This package is an example of a simple timer-triggered Azure Function.

## Example function implementation

The example timer-triggered Azure Function that runs every minute:

```rust
use azure_functions::bindings::TimerInfo;
use azure_functions::func;

#[func]
#[binding(name = "info", schedule = "0 */1 * * * *")]
pub fn timer(info: &TimerInfo) {
    info!("Hello every minute from Rust!");
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
AzureWebJobsScriptRoot=$SCRIPT_ROOT AzureWebJobsStorage=$CONNECTION_STRING dotnet run
```

Where `$SCRIPT_ROOT` above represents the path to the root directory created from running `cargo run` above and `$CONNECTION_STRING` is the Azure Storage connection string the Azure Functions host should use.

_Note: the syntax above works on macOS and Linux; on Windows, set the environment variables before running `dotnet run`._

## Invoke the `timer` function

The example function is automatically invoked by the Azure Functions Host when the timer expires.

Wait a minute and then check the Azure Functions Host output.

With any luck, you should see the following output:

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
