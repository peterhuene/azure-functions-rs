use azure_functions::{bindings::TimerInfo, func};

#[func]
#[binding(name = "info", schedule = "0 */1 * * * *")]
pub fn timer(info: &TimerInfo) {
    log::info!("Hello from Rust!");
    log::info!("Timer information: {:?}", info);
}
