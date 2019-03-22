use crate::bindings::EventHubMessage;
use crate::event_hub::{PartitionContext, SystemProperties};
use crate::rpc::protocol;
use crate::util::convert_from;
use chrono::{DateTime, Utc};
use serde_json::{from_str, Value};
use std::collections::HashMap;

const PARTITION_CONTEXT_KEY: &str = "PartitionContext";
const ENQUEUED_TIME_KEY: &str = "EnqueuedTimeUtc";
const OFFSET_KEY: &str = "Offset";
const PROPERTIES_KEY: &str = "Properties";
const SEQUENCE_NUMBER_KEY: &str = "SequenceNumber";
const SYSTEM_PROPERTIES_KEY: &str = "SystemProperties";

/// Represents an Event Hub trigger binding.
///
/// # Examples
///
/// ```rust
/// use azure_functions::{
///     bindings::EventHubTrigger,
///     func,
/// };
/// use log::warn;
///
/// #[func]
/// #[binding(name = "trigger", connection = "my_connection")]
/// pub fn log_event(trigger: EventHubTrigger) {
///     log::warn!("{:?}", trigger);
/// }
/// ```
#[derive(Debug)]
pub struct EventHubTrigger {
    /// The Event Hub message that triggered the function.
    pub message: EventHubMessage,
    /// The partition context information.
    pub partition_context: PartitionContext,
    /// The enqueued time in UTC.
    pub enqueued_time: DateTime<Utc>,
    /// The offset of the data relative to the Event Hub partition stream.
    pub offset: String,
    /// The user properties of the event data.
    pub properties: Value,
    /// The logical sequence number of the event.
    pub sequence_number: i64,
    /// The system properties of the event data.
    pub system_properties: SystemProperties,
}

impl EventHubTrigger {
    #[doc(hidden)]
    pub fn new(
        data: protocol::TypedData,
        metadata: &mut HashMap<String, protocol::TypedData>,
    ) -> Self {
        EventHubTrigger {
            message: data.into(),
            partition_context: from_str(
                metadata
                    .get(PARTITION_CONTEXT_KEY)
                    .expect("expected partition context")
                    .get_json(),
            )
            .expect("failed to deserialize partition context"),
            enqueued_time: convert_from(
                metadata
                    .get(ENQUEUED_TIME_KEY)
                    .expect("expected enqueued time"),
            )
            .expect("failed to convert enqueued time"),
            offset: metadata
                .get_mut(OFFSET_KEY)
                .expect("expected offset")
                .take_string(),
            properties: from_str(
                metadata
                    .get(PROPERTIES_KEY)
                    .expect("expected properties")
                    .get_json(),
            )
            .expect("failed to deserialize properties"),
            sequence_number: convert_from(
                metadata
                    .get(SEQUENCE_NUMBER_KEY)
                    .expect("expected sequence number"),
            )
            .expect("failed to convert sequence number"),
            system_properties: from_str(
                metadata
                    .get(SYSTEM_PROPERTIES_KEY)
                    .expect("expected system properties")
                    .get_json(),
            )
            .expect("failed to deserialize system properties"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_hub::RuntimeInformation;
    use serde_json::json;
    use std::str::FromStr;

    #[test]
    fn it_constructs() {
        const MESSAGE: &str = "hello world!";
        const ENQUEUED_TIME: &str = "2018-07-25T06:24:00+00:00";
        const RETRIEVAL_TIME: &str = "0001-01-01T00:00:00Z";
        const OFFSET: &str = "98765";
        const SEQUENCE_NUMBER: i64 = 12345;
        const PARTITION_ID: &str = "1";
        const OWNER: &str = "358d9b05-56fe-4549-bafb-e0e102b29b05";
        const EVENT_HUB_PATH: &str = "my_event_hub";
        const CONSUMER_GROUP: &str = "$Default";
        const USER_PROPERTY_NAME: &str = "property name";
        const USER_PROPERTY_VALUE: &str = "property value";
        const PARTITION_KEY: &str = "partition key";

        let mut data = protocol::TypedData::new();
        data.set_string(MESSAGE.to_string());

        let mut metadata = HashMap::new();

        let context = PartitionContext {
            consumer_group_name: CONSUMER_GROUP.to_string(),
            event_hub_path: EVENT_HUB_PATH.to_string(),
            partition_id: PARTITION_ID.to_string(),
            owner: OWNER.to_string(),
            runtime_information: RuntimeInformation {
                partition_id: PARTITION_ID.to_string(),
                last_sequence_number: SEQUENCE_NUMBER,
                last_enqueued_time: DateTime::<Utc>::from_str(ENQUEUED_TIME).unwrap(),
                last_enqueued_offset: Some(OFFSET.to_string()),
                retrieval_time: DateTime::<Utc>::from_str(RETRIEVAL_TIME).unwrap(),
            },
        };

        let properties = json!({ USER_PROPERTY_NAME: USER_PROPERTY_VALUE });

        let system_properties = SystemProperties {
            sequence_number: SEQUENCE_NUMBER,
            offset: OFFSET.to_string(),
            partition_key: Some(PARTITION_KEY.to_string()),
            enqueued_time: DateTime::<Utc>::from_str(ENQUEUED_TIME).unwrap(),
        };

        let mut value = protocol::TypedData::new();
        value.set_json(serde_json::to_string(&context).unwrap());
        metadata.insert(PARTITION_CONTEXT_KEY.to_string(), value);

        let mut value = protocol::TypedData::new();
        value.set_string(ENQUEUED_TIME.to_string());
        metadata.insert(ENQUEUED_TIME_KEY.to_string(), value);

        let mut value = protocol::TypedData::new();
        value.set_string(OFFSET.to_string());
        metadata.insert(OFFSET_KEY.to_string(), value);

        let mut value = protocol::TypedData::new();
        value.set_json(properties.to_string());
        metadata.insert(PROPERTIES_KEY.to_string(), value);

        let mut value = protocol::TypedData::new();
        value.set_int(SEQUENCE_NUMBER);
        metadata.insert(SEQUENCE_NUMBER_KEY.to_string(), value);

        let mut value = protocol::TypedData::new();
        value.set_json(serde_json::to_string(&system_properties).unwrap());
        metadata.insert(SYSTEM_PROPERTIES_KEY.to_string(), value);

        let trigger = EventHubTrigger::new(data, &mut metadata);

        assert_eq!(trigger.message.as_str().unwrap(), MESSAGE);
        assert_eq!(
            trigger.partition_context.consumer_group_name,
            CONSUMER_GROUP
        );
        assert_eq!(trigger.partition_context.event_hub_path, EVENT_HUB_PATH);
        assert_eq!(trigger.partition_context.partition_id, PARTITION_ID);
        assert_eq!(trigger.partition_context.owner, OWNER);
        assert_eq!(
            trigger.partition_context.runtime_information.partition_id,
            PARTITION_ID
        );
        assert_eq!(
            trigger
                .partition_context
                .runtime_information
                .last_sequence_number,
            SEQUENCE_NUMBER
        );
        assert_eq!(
            trigger
                .partition_context
                .runtime_information
                .last_enqueued_time
                .to_rfc3339(),
            ENQUEUED_TIME
        );
        assert_eq!(trigger.enqueued_time.to_rfc3339(), ENQUEUED_TIME);
        assert_eq!(trigger.offset, OFFSET);
        assert_eq!(
            trigger.properties,
            json! {{ USER_PROPERTY_NAME: USER_PROPERTY_VALUE }}
        );
        assert_eq!(trigger.sequence_number, SEQUENCE_NUMBER);
        assert_eq!(trigger.system_properties.sequence_number, SEQUENCE_NUMBER);
        assert_eq!(trigger.system_properties.offset, OFFSET);
        assert_eq!(
            trigger.system_properties.partition_key.unwrap(),
            PARTITION_KEY
        );
        assert_eq!(
            trigger.system_properties.enqueued_time.to_rfc3339(),
            ENQUEUED_TIME
        );
    }
}
