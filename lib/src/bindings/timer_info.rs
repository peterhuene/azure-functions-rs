use rpc::protocol;
use serde_json::from_str;
use timer::ScheduleStatus;

/// Represents the timer information from a timer trigger binding.
///
/// # Examples
///
/// A function that runs every 5 minutes:
///
/// ```rust
/// # #![feature(use_extern_macros)] extern crate azure_functions;
/// # #[macro_use] extern crate log;
/// use azure_functions::bindings::TimerInfo;
/// use azure_functions::func;
///
/// #[func]
/// #[binding(name = "info", schedule = "0 */5 * * * *")]
/// pub fn timer(info: &TimerInfo) {
///     info!("Rust Azure function ran!");
/// }
/// ```
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TimerInfo {
    /// The schedule status for the timer.
    /// If schedule monitoring is not enabled for the timer, this field will be `None`.
    pub schedule_status: Option<ScheduleStatus>,
    /// Determines if the timer invocation is due to a missed schedule occurrence.
    pub is_past_due: bool,
}

impl<'a> From<&'a protocol::TypedData> for TimerInfo {
    fn from(data: &'a protocol::TypedData) -> Self {
        if !data.has_json() {
            panic!("expected JSON data for timer trigger binding");
        }

        from_str(data.get_json()).expect("failed to parse timer JSON data")
    }
}
