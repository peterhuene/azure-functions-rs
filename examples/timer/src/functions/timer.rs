use azure_functions::bindings::TimerInfo;
use azure_functions::func;

#[func]
#[binding(name = "info", schedule = "0 */1 * * * *")]
pub fn timer(info: &TimerInfo) {
    info!("Hello from Rust!");
    info!("Timer information: {:?}", info);
}
