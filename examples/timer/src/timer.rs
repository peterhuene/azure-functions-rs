use azure_functions::bindings::TimerInfo;
use azure_functions::func;

// Example of an Azure Function that is invoked every 5 minutes
#[func]
#[binding(name = "info", schedule = "0 */5 * * * *")]
pub fn timer(info: &TimerInfo) {
    debug!("Info: {:?}", info);
}
