use chrono::{DateTime, FixedOffset, Utc};
use rpc::protocol;
use serde::{de::Error, Deserialize, Deserializer};
use serde_json::from_str;

/// Represents a timer binding schedule status.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ScheduleStatus {
    /// The last recorded schedule occurrence.
    #[serde(deserialize_with = "deserialize_datetime")]
    pub last: DateTime<Utc>,
    /// The expected next schedule occurrence.
    #[serde(deserialize_with = "deserialize_datetime")]
    pub next: DateTime<Utc>,
    /// The last time the timer record was updated.
    ///
    /// This is used to re-calculate `next` with the current schedule after a host restart.
    #[serde(deserialize_with = "deserialize_datetime")]
    pub last_updated: DateTime<Utc>,
}

fn deserialize_datetime<'a, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'a>,
{
    let mut s = String::deserialize(deserializer)?;

    // This exists because the Azure Functions Host serializes DateTime.MinValue without a timezone
    // However, chrono::DateTime requires one for DateTime<Utc>
    if s == "0001-01-01T00:00:00" {
        s += "Z";
    }

    s.parse::<DateTime<FixedOffset>>()
        .map_err(|e| Error::custom(format!("{}", e)))
        .map(|dt| dt.with_timezone(&Utc))
}

/// Represents the timer information from a timer trigger binding.
///
/// # Examples
///
/// A function that runs every 5 minutes:
///
/// ```rust
/// # #![feature(proc_macro)] extern crate azure_functions;
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
