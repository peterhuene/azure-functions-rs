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
