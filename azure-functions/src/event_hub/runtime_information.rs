use crate::util::deserialize_datetime;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents the approximate receiver runtime information for a logical partition of an Event Hub.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RuntimeInformation {
    /// The partition ID for a logical partition of an Event Hub.
    pub partition_id: String,
    /// The last sequence number of the event within the partition stream of the Event Hub.
    pub last_sequence_number: i64,
    /// The enqueued time (in UTC) of the last event.
    #[serde(
        rename = "LastEnqueuedTimeUtc",
        deserialize_with = "deserialize_datetime"
    )]
    pub last_enqueued_time: DateTime<Utc>,
    /// The offset of the last enqueued event.
    pub last_enqueued_offset: Option<String>,
    /// The time when the runtime information was retrieved.
    #[serde(deserialize_with = "deserialize_datetime")]
    pub retrieval_time: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::from_str;

    #[test]
    fn it_deserializes_from_json() {
        const JSON: &'static str = r#"{"PartitionId":"0","LastSequenceNumber":0,"LastEnqueuedTimeUtc":"0001-01-01T00:00:00","LastEnqueuedOffset":null,"RetrievalTime":"0001-01-01T00:00:00"}"#;

        let info: RuntimeInformation =
            from_str(JSON).expect("failed to parse runtime information JSON data");
        assert_eq!(info.partition_id, "0");
        assert_eq!(info.last_sequence_number, 0);
        assert_eq!(
            info.last_enqueued_time.to_rfc3339(),
            "0001-01-01T00:00:00+00:00"
        );
        assert_eq!(info.last_enqueued_offset, None);
        assert_eq!(
            info.retrieval_time.to_rfc3339(),
            "0001-01-01T00:00:00+00:00"
        );
    }
}
