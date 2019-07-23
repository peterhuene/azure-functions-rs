use azure_functions::{bindings::DurableActivityContext, durable::ActivityOutput, func};

#[func(name = "SayHello")]
pub async fn say_hello(_context: DurableActivityContext) -> ActivityOutput {
    // format!(
    //     "Hello {}!",
    //     context.input().as_str().expect("expected a string input")
    // )
    // .into()
    unimplemented!()
}
