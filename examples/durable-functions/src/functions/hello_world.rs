use azure_functions::{bindings::DurableOrchestrationContext, durable::OrchestrationOutput, func};
use log::warn;
use serde_json::Value;

#[func]
pub async fn hello_world(context: DurableOrchestrationContext) -> OrchestrationOutput {
    if !context.is_replaying() {
        warn!("Orchestration started at {}.", context.current_time());
    }

    let activities = vec![
        context.call_activity("say_hello", "Tokyo"),
        context.call_activity("say_hello", "London"),
        context.call_activity("say_hello", "Seattle"),
    ];

    if !context.is_replaying() {
        warn!("Joining all activities.");
    }

    let result: Vec<_> = context
        .join_all(activities)
        .await
        .into_iter()
        .map(|r| r.unwrap_or_else(|e| Value::from(format!("Activity failed: {}", e))))
        .collect();

    if !context.is_replaying() {
        warn!("Result is: {:#?}.", result);
    }

    let mut activities = vec![
        context.call_activity("say_hello", "Jakarta"),
        context.call_activity("say_hello", "Portland"),
        context.call_activity("say_hello", "New York"),
    ];

    if !context.is_replaying() {
        warn!("Selecting all activities.");
    }

    let mut completed = 0;

    while !activities.is_empty() {
        let (r, _, remaining) = context.select_all(activities).await;

        completed += 1;

        if !context.is_replaying() {
            warn!("Result #{} is: {:#?}.", completed, r);
        }

        activities = remaining;
    }

    if !context.is_replaying() {
        warn!("Orchestration completed at {}.", context.current_time());
    }

    result.into()
}
