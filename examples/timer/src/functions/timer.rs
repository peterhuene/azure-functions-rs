use azure_functions::{bindings::TimerInfo, func};

#[func]
pub fn timer(#[binding(schedule = "0 */1 * * * *")] info: TimerInfo) {
    log::info!("Hello from Rust!");
    log::info!("Timer information: {:?}", info);
}
