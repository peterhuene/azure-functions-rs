use crate::event_hub::RuntimeInformation;

/// Encapsulates information related to an Event Hubs partition.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PartitionContext {
    /// The name of the consumer group.
    pub consumer_group_name: String,
    /// The path of the event hub.
    pub event_hub_path: String,
    /// The partition ID for the context.
    pub partition_id: String,
    /// The host owner for the partition.
    pub owner: String,
    /// The approximate receiver runtime information for a logical partition of the Event Hub.
    pub runtime_information: RuntimeInformation,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::from_str;

    #[test]
    fn it_deserializes_from_json() {
        const JSON: &'static str = r#"{"ConsumerGroupName":"$Default","EventHubPath":"test","PartitionId":"0","Owner":"40eeeb3a-3491-4072-ba37-59ecdc330b6e","RuntimeInformation":{"PartitionId":"0","LastSequenceNumber":0,"LastEnqueuedTimeUtc":"0001-01-01T00:00:00","LastEnqueuedOffset":null,"RetrievalTime":"0001-01-01T00:00:00"}}"#;

        let context: PartitionContext =
            from_str(JSON).expect("failed to parse partition context JSON data");

        assert_eq!(context.consumer_group_name, "$Default");
        assert_eq!(context.event_hub_path, "test");
        assert_eq!(context.partition_id, "0");
        assert_eq!(context.owner, "40eeeb3a-3491-4072-ba37-59ecdc330b6e");
        assert_eq!(context.runtime_information.partition_id, "0");
        assert_eq!(context.runtime_information.last_sequence_number, 0);
        assert_eq!(
            context.runtime_information.last_enqueued_time.to_rfc3339(),
            "0001-01-01T00:00:00+00:00"
        );
        assert_eq!(context.runtime_information.last_enqueued_offset, None);
        assert_eq!(
            context.runtime_information.retrieval_time.to_rfc3339(),
            "0001-01-01T00:00:00+00:00"
        );
    }
}
