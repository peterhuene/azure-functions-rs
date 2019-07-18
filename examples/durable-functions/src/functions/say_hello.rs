use azure_functions::{bindings::DurableActivityContext, func};

#[func(name = "SayHello")]
pub async fn say_hello(_context: DurableActivityContext) {
    // context.set_output(format!(
    //     "Hello {}!",
    //     context
    //         .get_input()
    //         .as_str()
    //         .expect("expected a string input")
    // ));
    unimplemented!()
}
