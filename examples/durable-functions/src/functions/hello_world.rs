use azure_functions::{bindings::DurableOrchestrationContext, func};
use futures::future::join_all;

#[func(name = "HelloWorld")]
pub async fn hello_world(context: DurableOrchestrationContext) {
    // context.set_output(
    //     join_all(
    //         [
    //             context.call_activity("SayHello", "Tokyo"),
    //             context.call_activity("SayHello", "London"),
    //             context.call_activity("SayHello", "Seattle"),
    //         ]
    //         .into_iter(),
    //     )
    //     .await,
    // );
}
