use azure_functions::{bindings::DurableOrchestrationContext, durable::OrchestrationOutput, func};
use log::warn;
use serde_json::Value;

#[func]
pub async fn looping(context: DurableOrchestrationContext) -> OrchestrationOutput {
    let value = context.input.as_i64().expect("expected a number for input");

    if !context.is_replaying() {
        warn!("The current value is: {}.", value);
    }

    if value < 10 {
        return context.continue_as_new(value + 1, true);
    }

    if !context.is_replaying() {
        warn!("Loop has completed.");
    }

    Value::Null.into()
}
