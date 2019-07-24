use azure_functions::{bindings::DurableOrchestrationContext, durable::OrchestrationOutput, func};
use futures::future::join_all;

#[func(name = "HelloWorld")]
pub async fn hello_world(mut context: DurableOrchestrationContext) -> OrchestrationOutput {
    // join_all(
    //     [
    //         context.call_activity("SayHello", "Tokyo"),
    //         context.call_activity("SayHello", "London"),
    //         context.call_activity("SayHello", "Seattle"),
    //     ]
    //     .into_iter(),
    // )
    // .await
    // .map(|r| r.unwrap())
    // .into()
    unimplemented!()
}
