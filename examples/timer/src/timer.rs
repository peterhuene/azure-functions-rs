use azure_functions::bindings::TimerInfo;
use azure_functions::func;

#[func]
#[binding(name = "_info", schedule = "0 */1 * * * *")]
pub fn timer(_info: &TimerInfo) {
    info!("Hello every minute from Rust!");
}
