use azure_functions::{bindings::DurableOrchestrationContext, durable::OrchestrationOutput, func};
use log::warn;
use serde_json::Value;

#[func]
pub async fn hello_world(mut context: DurableOrchestrationContext) -> OrchestrationOutput {
    if !context.is_replaying() {
        warn!("Orchestration started at {}.", context.current_time());
    }

    let activities = vec![
        context.call_activity("say_hello", "Tokyo"),
        context.call_activity("say_hello", "London"),
        context.call_activity("say_hello", "Seattle"),
    ];

    let result: Vec<_> = context
        .join_all(activities)
        .await
        .into_iter()
        .map(|r| r.unwrap_or_else(|e| Value::from(format!("Activity failed: {}", e))))
        .collect();

    if !context.is_replaying() {
        warn!("Orchestration completed at {}.", context.current_time());
    }

    result.into()
}
