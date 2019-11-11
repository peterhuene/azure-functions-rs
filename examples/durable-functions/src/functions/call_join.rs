use azure_functions::{bindings::DurableOrchestrationContext, func};
use log::{error, info};
use serde_json::Value;

#[func]
pub async fn call_join(context: DurableOrchestrationContext) {
    match context
        .call_sub_orchestrator("join", None, Value::Null)
        .await
    {
        Ok(output) => info!("The output of the sub orchestration was: {}", output),
        Err(e) => error!("The sub orchestration failed: {}", e),
    };
}
