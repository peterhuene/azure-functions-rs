use bindings::Trigger;
use rpc::protocol;
use serde_json::from_str;
use std::collections::HashMap;
use timer::ScheduleStatus;

/// Represents the timer information from a timer trigger binding.
///
/// # Examples
///
/// A function that runs every 5 minutes:
///
/// ```rust
/// # extern crate azure_functions;
/// # #[macro_use] extern crate log;
/// use azure_functions::bindings::TimerInfo;
/// use azure_functions::func;
///
/// #[func]
/// #[binding(name = "_info", schedule = "0 */5 * * * *")]
/// pub fn timer(_info: &TimerInfo) {
///     info!("Rust Azure function ran!");
/// }
/// ```
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TimerInfo {
    /// The schedule status for the timer.
    ///
    /// If schedule monitoring is not enabled for the timer, this field will be `None`.
    pub schedule_status: Option<ScheduleStatus>,
    /// Determines if the timer invocation is due to a missed schedule occurrence.
    pub is_past_due: bool,
}

impl From<protocol::TypedData> for TimerInfo {
    fn from(data: protocol::TypedData) -> Self {
        if !data.has_json() {
            panic!("expected JSON data for timer trigger binding");
        }

        from_str(data.get_json()).expect("failed to parse timer JSON data")
    }
}

impl Trigger for TimerInfo {
    fn read_metadata(&mut self, _: &mut HashMap<String, protocol::TypedData>) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_has_json_data() {
        const JSON: &'static str = r#"{"Schedule":{},"ScheduleStatus":{"Last":"0001-01-01T00:00:00","Next":"2018-07-24T23:24:00-07:00","LastUpdated":"0001-01-01T00:00:00"},"IsPastDue":true}"#;

        let mut data = protocol::TypedData::new();
        data.set_json(JSON.to_string());

        let info: TimerInfo = data.into();

        assert!(info.schedule_status.is_some());
        assert!(info.is_past_due);

        let status = info.schedule_status.as_ref().unwrap();
        assert_eq!(status.last.to_rfc3339(), "0001-01-01T00:00:00+00:00");
        assert_eq!(status.next.to_rfc3339(), "2018-07-25T06:24:00+00:00");
        assert_eq!(
            status.last_updated.to_rfc3339(),
            "0001-01-01T00:00:00+00:00"
        );
    }
}
