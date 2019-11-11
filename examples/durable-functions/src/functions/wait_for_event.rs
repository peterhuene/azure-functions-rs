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
