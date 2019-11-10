use azure_functions::{bindings::DurableOrchestrationContext, func};
use log::warn;

#[func]
pub async fn looping(context: DurableOrchestrationContext) {
    let value = context.input.as_i64().expect("expected a number for input");

    if !context.is_replaying() {
        warn!("The current value is: {}.", value);
    }

    if value < 10 {
        context.continue_as_new(value + 1, true);
        return;
    }

    if !context.is_replaying() {
        warn!("Loop has completed.");
    }
}
