use azure_functions::{bindings::DurableEntityContext, func};

#[func]
pub fn counter(context: DurableEntityContext) {
    let current = context.state().map_or(0, |v| v.as_i64().unwrap_or(0));

    match context.operation_name() {
        "add" => context.set_state(current + context.input().as_i64().unwrap_or(0)),
        "reset" => context.set_state(0),
        "delete" => context.delete_state(),
        "get" => context.set_return(current),
        _ => {}
    };
}
