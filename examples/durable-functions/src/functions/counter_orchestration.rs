use azure_functions::{bindings::DurableOrchestrationContext, func};
use log::info;
use serde_json::Value;

#[func]
pub async fn counter_orchestration(context: DurableOrchestrationContext) {
    let counter = context
        .call_entity("counter", "myCounter", "get", Value::Null)
        .await
        .expect("expected the counter value")
        .as_i64()
        .expect("expected the counter to be an integer");

    if !context.is_replaying() {
        info!("Counter is: {}", counter);
    }

    if counter < 10 {
        if !context.is_replaying() {
            info!("Incrementing counter.");
        }

        let _ = context.call_entity("counter", "myCounter", "add", 1).await;
    }
}
