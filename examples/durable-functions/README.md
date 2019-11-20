# Durable Functions for Rust Example

This project is an example of using Durable Functions for Rust.

## Example function implementations

A HTTP-triggered function used to start orchestrations.  This function takes a `function` query parameter as the name of the orchestration function to call:

```rust
use azure_functions::{
    bindings::{DurableOrchestrationClient, HttpRequest, HttpResponse},
    func,
};
use serde_json::Value;

#[func]
pub async fn start(req: HttpRequest, client: DurableOrchestrationClient) -> HttpResponse {
    match client
        .start_new(
            req.query_params
                .get("function")
                .expect("expected a function parameter"),
            None,
            Value::Null,
        )
        .await
    {
        Ok(data) => data.into(),
        Err(e) => format!("Failed to start orchestration: {}", e).into(),
    }
}
```

An orchestration function that calls three activities and waits for all the activities to complete:

```rust
use azure_functions::{bindings::DurableOrchestrationContext, durable::OrchestrationOutput, func};
use log::{error, info};
use serde_json::Value;

#[func]
pub async fn join(context: DurableOrchestrationContext) -> OrchestrationOutput {
    if !context.is_replaying() {
        info!("Orchestration started at {}.", context.current_time());
    }

    let activities = vec![
        context.call_activity("say_hello", "Tokyo"),
        context.call_activity("say_hello", "London"),
        context.call_activity("say_hello", "Seattle"),
    ];

    if !context.is_replaying() {
        info!("Joining all activities.");
    }

    context.set_custom_status("Waiting for all activities to complete.");

    let result: Value = context
        .join_all(activities)
        .await
        .into_iter()
        .filter_map(|r| {
            r.map(Some).unwrap_or_else(|e| {
                error!("Activity failed: {}", e);
                None
            })
        })
        .collect::<Vec<_>>()
        .into();

    if !context.is_replaying() {
        info!(
            "Orchestration completed at {} with result: {}.",
            context.current_time(),
            result
        );
    }

    context.set_custom_status("All activities have completed.");

    result.into()
}
```

An orchestration function that calls three activities and logs the output as each activity completes:

```rust
use azure_functions::{bindings::DurableOrchestrationContext, func};
use log::{error, info};

#[func]
pub async fn select(context: DurableOrchestrationContext) {
    if !context.is_replaying() {
        info!("Orchestration started at {}.", context.current_time());
    }

    let mut activities = vec![
        context.call_activity("say_hello", "Jakarta"),
        context.call_activity("say_hello", "Portland"),
        context.call_activity("say_hello", "New York"),
    ];

    if !context.is_replaying() {
        info!("Selecting all activities.");
    }

    let mut completed = 0;

    while !activities.is_empty() {
        context.set_custom_status(format!(
            "Waiting on {} remaining activities.",
            activities.len()
        ));

        let (r, _, remaining) = context.select_all(activities).await;

        completed += 1;

        if !context.is_replaying() {
            match r {
                Ok(output) => info!("Activity #{} completed with output: {}", completed, output),
                Err(e) => error!("Activity #{} failed: {}", completed, e),
            };
        }

        activities = remaining;
    }

    context.set_custom_status("All activities have completed.");

    if !context.is_replaying() {
        info!("Orchestration completed at {}.", context.current_time(),);
    }
}
```

An activity function that takes a string input and outputs a formatted message:

```rust
use azure_functions::{bindings::DurableActivityContext, durable::ActivityOutput, func};

#[func]
pub fn say_hello(context: DurableActivityContext) -> ActivityOutput {
    format!(
        "Hello {}!",
        context.input.as_str().expect("expected a string input")
    )
    .into()
}
```

An orchestration function that calls a sub-orchestration (the `join` orchestration from above) and logs the output:

```rust
use azure_functions::{bindings::DurableOrchestrationContext, func};
use log::{error, info};
use serde_json::Value;

#[func]
pub async fn call_join(context: DurableOrchestrationContext) {
    match context
        .call_sub_orchestrator("join", None, Value::Null)
        .await
    {
        Ok(output) => info!("The output of the sub orchestration was: {}", output),
        Err(e) => error!("The sub orchestration failed: {}", e),
    };
}
```

An orchestration function that waits for an external event named `event`:

```rust
use azure_functions::{bindings::DurableOrchestrationContext, durable::OrchestrationOutput, func};
use log::info;

#[func]
pub async fn wait_for_event(context: DurableOrchestrationContext) -> OrchestrationOutput {
    if !context.is_replaying() {
        info!("Waiting for event named 'event'.");
    }

    let v = context.wait_for_event("event").await.unwrap();

    if !context.is_replaying() {
        info!("Event was raised with value: {}.", v.as_str().unwrap());
    }

    v.into()
}
```

A HTTP-triggered function that can raise the external event.  The function takes three query parameters: `id` (the instance identifier of the orchestration), `name` (the name of the event to raise), and `value` (the value returned to the waiting orchestration function):

```rust
use azure_functions::{
    bindings::{DurableOrchestrationClient, HttpRequest, HttpResponse},
    func,
};

#[func]
pub async fn raise_event(mut req: HttpRequest, client: DurableOrchestrationClient) -> HttpResponse {
    let value = req
        .query_params
        .remove("value")
        .expect("expected a 'value' parameter");

    let id = req
        .query_params
        .get("id")
        .expect("expected a 'id' parameter");

    let name = req
        .query_params
        .get("name")
        .expect("expected a 'name' parameter");

    match client.raise_event(id, name, value).await {
        Ok(_) => format!("Raised event named '{}'.", name).into(),
        Err(e) => format!("Failed to raise event named '{}': {}", name, e).into(),
    }
}
```

An orchestration function that waits for a timer:

```rust
use azure_functions::{bindings::DurableOrchestrationContext, func};
use chrono::Duration;
use log::info;

#[func]
pub async fn timer(context: DurableOrchestrationContext) {
    if !context.is_replaying() {
        info!("Waiting 5 seconds.");
    }

    context
        .create_timer(context.current_time() + Duration::seconds(5))
        .await;

    if !context.is_replaying() {
        info!("Timer has fired.");
    }
}
```

A HTTP-triggered function that starts a "looping" orchestration function:

```rust
use azure_functions::{
    bindings::{DurableOrchestrationClient, HttpRequest, HttpResponse},
    func,
};

#[func]
pub async fn start_looping(_req: HttpRequest, client: DurableOrchestrationClient) -> HttpResponse {
    match client.start_new("looping", None, 0).await {
        Ok(data) => data.into(),
        Err(e) => format!("Failed to start orchestration: {}", e).into(),
    }
}
```

An orchestration function that loops by continuing the orchestration as new:

```rust
use azure_functions::{bindings::DurableOrchestrationContext, func};
use log::info;

#[func]
pub async fn looping(context: DurableOrchestrationContext) {
    let value = context.input.as_i64().expect("expected a number for input");

    if !context.is_replaying() {
        info!("The current value is: {}.", value);
    }

    if value < 10 {
        context.continue_as_new(value + 1, true);
        return;
    }

    if !context.is_replaying() {
        info!("Loop has completed.");
    }
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

# Invoking the orchestration functions

## The `join` orchestration function

Invoke the `start` function to run the `join` orchestration:

```
$ curl http://localhost:8080/api/start\?function\=join
```

This will output various URLs that can be used to query the status of or send an event to the orchestation.

In the Azure Functions Host output, you should see a message showing the completion of the orchestration:

```
[11/12/19 2:36:40 AM] Orchestration completed at 2019-11-12 02:36:40.137035 UTC with result: ["Hello Tokyo!","Hello London!","Hello Seattle!"]
```

## The `select` orchestration function

Invoke the `start` function to run the `select` orchestration:

```
$ curl http://localhost:8080/api/start\?function\=select
```

This will output various URLs that can be used to query the status of or send an event to the orchestation.

In the Azure Functions Host output, you should see a message showing the completion of each individual activity (the order is nondeterministic):

```
[11/12/19 2:40:04 AM] Activity #1 completed with output: "Hello Portland!"
[11/12/19 2:40:04 AM] Activity #2 completed with output: "Hello New York!"
[11/12/19 2:40:04 AM] Activity #3 completed with output: "Hello Jakarta!"
```

## The `call_join` orchestration function

Invoke the `start` function to run the `call_join` orchestration:

```
$ curl http://localhost:8080/api/start\?function\=call_join
```

This will output various URLs that can be used to query the status of or send an event to the orchestation.

In the Azure Functions Host output, you should see a message showing the completion of the orchestration:

```
[11/12/19 2:43:09 AM] The output of the sub orchestration was: ["Hello Tokyo!","Hello London!","Hello Seattle!"]
```

## The `wait_for_event` orchestration function

Invoke the `start` function to run the `wait_for_event` orchestration:

```
$ curl http://localhost:8080/api/start\?function\=wait_for_event
```

In the Azure Functions Host output, you should see a message showing that the orchestration is waiting for an event:

```
[11/12/19 2:45:20 AM] Waiting for event named 'event'.
```

Raise an event named `event` to continue the orchestration function (use the `id` from the curl output above for the `$ID` variable):

```
$ curl http://localhost:8080/api/raise_event\?id\=$ID\&name\=event\&value\=hello%20world
```

In the Azure Functions Host output, you should see a message showing that the orchestration received the event:

```
[11/12/19 2:48:04 AM] Event was raised with value: hello world.
```

## The `looping` orchestration function

Invoke the `start_looping` function to run the `looping` orchestration:

```
$ curl http://localhost:8080/api/start_looping
```

In the Azure Functions Host output, you should see messages showing that the loop counter increased from 0 to 10:

```
[11/12/19 2:50:14 AM] The current value is: 0.
[11/12/19 2:50:14 AM] The current value is: 1.
[11/12/19 2:50:14 AM] The current value is: 2.
[11/12/19 2:50:14 AM] The current value is: 3.
[11/12/19 2:50:14 AM] The current value is: 4.
[11/12/19 2:50:14 AM] The current value is: 5.
[11/12/19 2:50:14 AM] The current value is: 6.
[11/12/19 2:50:14 AM] The current value is: 7.
[11/12/19 2:50:14 AM] The current value is: 8.
[11/12/19 2:50:14 AM] The current value is: 9.
[11/12/19 2:50:14 AM] The current value is: 10.
[11/12/19 2:50:14 AM] Loop has completed.
```