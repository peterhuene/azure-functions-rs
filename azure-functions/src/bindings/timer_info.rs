use crate::rpc::protocol;
use crate::timer::ScheduleStatus;
use serde_derive::Deserialize;
use serde_json::from_str;
use std::collections::HashMap;

/// Represents the timer information from a timer trigger binding.
///
/// The following binding attributes are supported:
///
/// | Name             | Description                                                                                                                                                                                                                        |
/// |------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
/// | `name`           | The name of the parameter being bound.                                                                                                                                                                                             |
/// | `schedule`       | The CRON expression or a TimeSpan value for the timer. A TimeSpan can be used only for a function app that runs on an App Service Plan.                                                                                            |
/// | `run_on_startup` | If `true`, the function is invoked when the runtime starts. It should rarely, if ever, be set to `true` in production as the function will be invoked on runtime restarts and scale outs.                                          |
/// | `use_monitor`    | Set to `true` or `false` to indicate whether the schedule should be monitored. Schedule monitoring persists schedule occurrences to aid in ensuring the schedule is maintained correctly even when function app instances restart. |
///
/// # Examples
///
/// A function that runs every 5 minutes:
///
/// ```rust
/// use azure_functions::bindings::TimerInfo;
/// use azure_functions::func;
/// use log::warn;
///
/// #[func]
/// #[binding(name = "_info", schedule = "0 */5 * * * *")]
/// pub fn timer(_info: TimerInfo) {
///     warn!("Rust Azure function ran!");
/// }
/// ```
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TimerInfo {
    /// The schedule status for the timer.
    ///
    /// If schedule monitoring is not enabled for the timer, this field will be `None`.
    pub schedule_status: ScheduleStatus,
    /// Determines if the timer invocation is due to a missed schedule occurrence.
    pub is_past_due: bool,
}

impl TimerInfo {
    #[doc(hidden)]
    pub fn new(data: protocol::TypedData, _: &mut HashMap<String, protocol::TypedData>) -> Self {
        if !data.has_json() {
            panic!("expected JSON data for timer trigger binding");
        }

        from_str(data.get_json()).expect("failed to parse timer JSON data")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_has_json_data() {
        const JSON: &'static str = r#"{"ScheduleStatus":{"Last":"0001-01-01T00:00:00","Next":"2018-07-24T23:24:00-07:00","LastUpdated":"0001-01-01T00:00:00"},"IsPastDue":true}"#;

        let mut data = protocol::TypedData::new();
        data.set_json(JSON.to_string());

        let mut metadata = HashMap::new();

        let info = TimerInfo::new(data, &mut metadata);

        assert!(info.is_past_due);

        assert_eq!(
            info.schedule_status.last.to_rfc3339(),
            "0001-01-01T00:00:00+00:00"
        );
        assert_eq!(
            info.schedule_status.next.to_rfc3339(),
            "2018-07-25T06:24:00+00:00"
        );
        assert_eq!(
            info.schedule_status.last_updated.to_rfc3339(),
            "0001-01-01T00:00:00+00:00"
        );
    }
}
