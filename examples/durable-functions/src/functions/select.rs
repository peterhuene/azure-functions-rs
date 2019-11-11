use azure_functions::{bindings::DurableOrchestrationContext, func};
use log::{error, info};

#[func]
pub async fn select(context: DurableOrchestrationContext) {
    if !context.is_replaying() {
        info!("Orchestration started at {}.", context.current_time());
    }

    let mut activities = vec![
        context.call_activity("say_hello", "Jakarta"),
        context.call_activity("say_hello", "Portland"),
        context.call_activity("say_hello", "New York"),
    ];

    if !context.is_replaying() {
        info!("Selecting all activities.");
    }

    let mut completed = 0;

    while !activities.is_empty() {
        context.set_custom_status(format!(
            "Waiting on {} remaining activities.",
            activities.len()
        ));

        let (r, _, remaining) = context.select_all(activities).await;

        completed += 1;

        if !context.is_replaying() {
            match r {
                Ok(output) => info!("Activity #{} completed with output: {}", completed, output),
                Err(e) => error!("Activity #{} failed: {}", completed, e),
            };
        }

        activities = remaining;
    }

    context.set_custom_status("All activities have completed.");

    if !context.is_replaying() {
        info!("Orchestration completed at {}.", context.current_time(),);
    }
}
