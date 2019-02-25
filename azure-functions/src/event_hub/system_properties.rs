use crate::util::deserialize_datetime;
use chrono::{DateTime, Utc};

/// Represents properties that are set by the Event Hubs service.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SystemProperties {
    /// The logical sequence number of the event within the partition stream of the Event Hub.
    pub sequence_number: i64,
    /// The data relative to the Event Hub partition stream.
    pub offset: String,
    /// The partition key of the corresponding partition.
    pub partition_key: Option<String>,
    /// The enqueuing time of the message time in UTC.
    #[serde(rename = "EnqueuedTimeUtc", deserialize_with = "deserialize_datetime")]
    pub enqueued_time: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::from_str;

    #[test]
    fn it_deserializes_from_json() {
        const JSON: &'static str = r#"{"SequenceNumber":3,"Offset":"152","PartitionKey":null,"EnqueuedTimeUtc":"2019-02-22T04:43:55.305Z"}"#;

        let properties: SystemProperties =
            from_str(JSON).expect("failed to parse system properties JSON data");
        assert_eq!(properties.sequence_number, 3);
        assert_eq!(properties.offset, "152");
        assert_eq!(properties.partition_key, None);
        assert_eq!(
            properties.enqueued_time.to_rfc3339(),
            "2019-02-22T04:43:55.305+00:00"
        );
    }
}
