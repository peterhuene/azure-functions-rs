use chrono::{DateTime, FixedOffset, Utc};
use serde::{de::Error, Deserialize, Deserializer};

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::from_str;

    #[test]
    fn it_deserializes_from_json() {
        const JSON: &'static str = r#"{"Last":"0001-01-01T00:00:00","Next":"2018-07-24T23:24:00-07:00","LastUpdated":"2018-07-28T02:00:32+00:00"}"#;

        let status: ScheduleStatus =
            from_str(JSON).expect("failed to parse schedule status JSON data");
        assert_eq!(status.last.to_rfc3339(), "0001-01-01T00:00:00+00:00");
        assert_eq!(status.next.to_rfc3339(), "2018-07-25T06:24:00+00:00");
        assert_eq!(
            status.last_updated.to_rfc3339(),
            "2018-07-28T02:00:32+00:00"
        );
    }
}
