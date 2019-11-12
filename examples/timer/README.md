# Example Timer Azure Function

This project is an example of a simple timer-triggered Azure Function.

## Example function implementation

The example timer-triggered Azure Function that runs every minute:

```rust
use azure_functions::{bindings::TimerInfo, func};

#[func]
#[binding(name = "info", schedule = "0 */1 * * * *")]
pub fn timer(info: TimerInfo) {
    log::info!("Hello from Rust!");
    log::info!("Timer information: {:?}", info);
}
```

# Running the example locally

Because this example relies on Azure Storage to function, the `AzureWebJobsStorage`
setting must be set to a connection string that the Azure Functions Host will use for
the default storage connection.

Add a setting for `AzureWebJobsStorage` into `local.settings.json`:

```
$ func settings add AzureWebJobsStorage <storage_connection_string>
```

You may encrypt `local.settings.json`, if desired:

```
$ func settings encrypt
```

Finally, start the Azure Functions application:

```
$ cargo func run
```

# Invoking the functions

## Invoke the `timer` function

The example function is automatically invoked by the Azure Functions Host when the timer expires.

Wait a minute and then check the Azure Functions Host output.

With any luck, you should see the following output from the Azure Functions Host:

```
nfo: Function.timer[0]
      Executing 'Functions.timer' (Reason='Timer fired at 2018-11-27T01:19:59.9935861+00:00', Id=2201c737-f12f-4e82-bdf2-f21969d29305)
info: Function.timer.User[0]
      Hello from Rust!
info: Function.timer.User[0]
      Timer information: TimerInfo { schedule_status: Some(ScheduleStatus { last: 0001-01-01T00:00:00Z, next: 2018-11-27T01:20:00Z, last_updated: 0001-01-01T00:00:00Z }), is_past_due: false }
info: Function.timer[0]
      Executed 'Functions.timer' (Succeeded, Id=2201c737-f12f-4e82-bdf2-f21969d29305)
```
