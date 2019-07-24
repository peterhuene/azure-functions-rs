use azure_functions::{bindings::DurableOrchestrationContext, durable::OrchestrationOutput, func};
use futures::future::join_all;

#[func]
pub async fn hello_world(mut context: DurableOrchestrationContext) -> OrchestrationOutput {
    // join_all(
    //     [
    //         context.call_activity("say_hello", "Tokyo"),
    //         context.call_activity("say_hello", "London"),
    //         context.call_activity("say_hello", "Seattle"),
    //     ]
    //     .into_iter(),
    // )
    // .await
    // .map(|r| r.unwrap())
    // .into()
    unimplemented!()
}
