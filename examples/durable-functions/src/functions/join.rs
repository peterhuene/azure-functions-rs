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
