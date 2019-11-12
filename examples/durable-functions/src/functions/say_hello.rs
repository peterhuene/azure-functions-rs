use azure_functions::{bindings::DurableActivityContext, durable::ActivityOutput, func};

#[func]
pub fn say_hello(context: DurableActivityContext) -> ActivityOutput {
    format!(
        "Hello {}!",
        context.input.as_str().expect("expected a string input")
    )
    .into()
}
