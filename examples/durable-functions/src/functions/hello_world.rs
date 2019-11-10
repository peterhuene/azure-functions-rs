use azure_functions::{bindings::DurableOrchestrationContext, durable::OrchestrationOutput, func};
use log::{error, info};
use serde_json::Value;

#[func]
pub async fn hello_world(context: DurableOrchestrationContext) -> OrchestrationOutput {
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
        info!("Result is: {}.", result);
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

    if !context.is_replaying() {
        info!("Orchestration completed at {}.", context.current_time());
    }

    result.into()
}
