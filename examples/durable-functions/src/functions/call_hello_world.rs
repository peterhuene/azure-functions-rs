use azure_functions::{bindings::DurableOrchestrationContext, func};
use log::{error, warn};
use serde_json::Value;

#[func]
pub async fn call_hello_world(context: DurableOrchestrationContext) {
    match context
        .call_sub_orchestrator("hello_world", None, Value::Null)
        .await
    {
        Ok(output) => warn!("The output of the sub orchestration was: {}", output),
        Err(e) => error!("The sub orchestration failed: {}", e),
    };
}
