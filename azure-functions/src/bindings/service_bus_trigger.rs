use crate::bindings::ServiceBusMessage;
use crate::rpc::protocol;
use crate::util::convert_from;
use chrono::{DateTime, Utc};
use serde_json::{from_str, Map, Value};
use std::collections::HashMap;

const DELIVERY_COUNT_KEY: &str = "DeliveryCount";
const DEAD_LETTER_SOURCE_KEY: &str = "DeadLetterSource";
const EXPIRATION_TIME_KEY: &str = "ExpiresAtUtc";
const ENQUEUED_TIME_KEY: &str = "EnqueuedTimeUtc";
const MESSAGE_ID_KEY: &str = "MessageId";
const CONTENT_TYPE_KEY: &str = "ContentType";
const REPLY_TO_KEY: &str = "ReplyTo";
const SEQUENCE_NUMBER_KEY: &str = "SequenceNumber";
const TO_KEY: &str = "To";
const LABEL_KEY: &str = "Label";
const CORRELATION_ID_KEY: &str = "CorrelationId";
const USER_PROPERTIES_KEY: &str = "UserProperties";

/// Represents a service bus trigger binding.
///
/// The following binding attributes are supported:
///
/// | Name                | Description                                                                                                                               |
/// |---------------------|-------------------------------------------------------------------------------------------------------------------------------------------|
/// | `name`              | The name of the parameter being bound.                                                                                                    |
/// | `queue_name`        | The name of the queue to monitor. Use only if monitoring a queue, not for a topic.                                                        |
/// | `topic_name`        | The name of the topic to monitor. Use only if monitoring a topic, not for a queue.                                                        |
/// | `subscription_name` | The name of the subscription to monitor. Use only if monitoring a topic, not for a queue.                                                 |
/// | `connection`        | The name of an app setting that contains the Service Bus connection string to use for this binding. Defaults to `AzureWebJobsServiceBus`. |
///
/// # Examples
///
/// An example that logs a message when a message is posted to a queue:
///
/// ```rust
/// use azure_functions::{
///     bindings::ServiceBusTrigger,
///     func,
/// };
///
/// #[func]
/// #[binding(name = "trigger", queue_name = "example", connection = "connection")]
/// pub fn log_message(trigger: ServiceBusTrigger) {
///     log::warn!("{}", trigger.message.as_str().unwrap());
/// }
/// ```
///
/// An example that logs a message when a message is posted to a topic and subscription:
///
/// ```rust
/// use azure_functions::{bindings::ServiceBusTrigger, func};
///
/// #[func]
/// #[binding(
///     name = "trigger",
///     topic_name = "mytopic",
///     subscription_name = "mysubscription",
///     connection = "connection"
/// )]
/// pub fn log_topic_message(trigger: ServiceBusTrigger) {
///     log::warn!("{}", trigger.message.as_str().unwrap());
/// }
/// ```
#[derive(Debug)]
pub struct ServiceBusTrigger {
    /// The message that triggered the function.
    pub message: ServiceBusMessage,
    /// The number of deliveries.
    pub delivery_count: i32,
    /// The dead letter source.
    pub dead_letter_source: Option<String>,
    /// The time that the message expires.
    pub expiration_time: DateTime<Utc>,
    /// The time that the message was enqueued.
    pub enqueued_time: DateTime<Utc>,
    /// The user-defined value that Service Bus can use to identify duplicate messages, if enabled.
    pub message_id: String,
    /// The content type identifier utilized by the sender and receiver for application specific logic.
    pub content_type: Option<String>,
    /// The reply to queue address.
    pub reply_to: Option<String>,
    /// The unique number assigned to a message by the Service Bus.
    pub sequence_number: i64,
    /// The send to address.
    pub to: Option<String>,
    /// The application specific label.
    pub label: Option<String>,
    /// The correlation ID.
    pub correlation_id: Option<String>,
    /// The application specific message properties.
    pub user_properties: Map<String, Value>,
}

impl ServiceBusTrigger {
    #[doc(hidden)]
    pub fn new(
        data: protocol::TypedData,
        metadata: &mut HashMap<String, protocol::TypedData>,
    ) -> Self {
        ServiceBusTrigger {
            message: data.into(),
            delivery_count: convert_from(
                metadata
                    .get(DELIVERY_COUNT_KEY)
                    .expect("expected a delivery count"),
            )
            .expect("failed to convert delivery count"),
            dead_letter_source: metadata
                .get_mut(DEAD_LETTER_SOURCE_KEY)
                .map(protocol::TypedData::take_string),
            expiration_time: convert_from(
                metadata
                    .get(EXPIRATION_TIME_KEY)
                    .expect("expected an expiration time"),
            )
            .expect("failed to convert expiration time"),
            enqueued_time: convert_from(
                metadata
                    .get(ENQUEUED_TIME_KEY)
                    .expect("expected an enqueued time"),
            )
            .expect("failed to convert enqueued time"),
            message_id: metadata
                .get_mut(MESSAGE_ID_KEY)
                .expect("expected a message id")
                .take_string(),
            content_type: metadata
                .get_mut(CONTENT_TYPE_KEY)
                .map(protocol::TypedData::take_string),
            reply_to: metadata
                .get_mut(REPLY_TO_KEY)
                .map(protocol::TypedData::take_string),
            sequence_number: convert_from(
                metadata
                    .get(SEQUENCE_NUMBER_KEY)
                    .expect("expected a sequence number"),
            )
            .expect("failed to convert sequence number"),
            to: metadata
                .get_mut(TO_KEY)
                .map(protocol::TypedData::take_string),
            label: metadata
                .get_mut(LABEL_KEY)
                .map(protocol::TypedData::take_string),
            correlation_id: metadata
                .get_mut(CORRELATION_ID_KEY)
                .map(protocol::TypedData::take_string),
            user_properties: from_str(
                metadata
                    .get(USER_PROPERTIES_KEY)
                    .map(protocol::TypedData::get_json)
                    .unwrap_or("{}"),
            )
            .expect("failed to convert user properties"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_constructs() {
        const DELIVERY_COUNT: i32 = 42;
        const DEAD_LETTER_SOURCE: &str = "dead letter source";
        const MESSAGE_ID: &str = "message id";
        const CONTENT_TYPE: &str = "content type";
        const REPLY_TO: &str = "reply to";
        const SEQUENCE_NUMBER: i64 = 12345678;
        const TO: &str = "to";
        const LABEL: &str = "label";
        const CORRELATION_ID: &str = "correlation id";
        const USER_PROPERTIES: &str = r#"{ "hello": "world" }"#;
        const MESSAGE: &'static str = "\"hello world\"";
        let now = Utc::now();

        let mut data = protocol::TypedData::new();
        data.set_json(MESSAGE.to_string());

        let mut metadata = HashMap::new();

        let mut value = protocol::TypedData::new();
        value.set_int(DELIVERY_COUNT.into());
        metadata.insert(DELIVERY_COUNT_KEY.to_string(), value);

        let mut value = protocol::TypedData::new();
        value.set_string(DEAD_LETTER_SOURCE.to_string());
        metadata.insert(DEAD_LETTER_SOURCE_KEY.to_string(), value);

        let mut value = protocol::TypedData::new();
        value.set_string(now.to_rfc3339());
        metadata.insert(EXPIRATION_TIME_KEY.to_string(), value);

        let mut value = protocol::TypedData::new();
        value.set_string(now.to_rfc3339());
        metadata.insert(ENQUEUED_TIME_KEY.to_string(), value);

        let mut value = protocol::TypedData::new();
        value.set_string(MESSAGE_ID.to_string());
        metadata.insert(MESSAGE_ID_KEY.to_string(), value);

        let mut value = protocol::TypedData::new();
        value.set_string(CONTENT_TYPE.to_string());
        metadata.insert(CONTENT_TYPE_KEY.to_string(), value);

        let mut value = protocol::TypedData::new();
        value.set_string(REPLY_TO.to_string());
        metadata.insert(REPLY_TO_KEY.to_string(), value);

        let mut value = protocol::TypedData::new();
        value.set_int(SEQUENCE_NUMBER);
        metadata.insert(SEQUENCE_NUMBER_KEY.to_string(), value);

        let mut value = protocol::TypedData::new();
        value.set_string(TO.to_string());
        metadata.insert(TO_KEY.to_string(), value);

        let mut value = protocol::TypedData::new();
        value.set_string(LABEL.to_string());
        metadata.insert(LABEL_KEY.to_string(), value);

        let mut value = protocol::TypedData::new();
        value.set_string(CORRELATION_ID.to_string());
        metadata.insert(CORRELATION_ID_KEY.to_string(), value);

        let mut value = protocol::TypedData::new();
        value.set_json(USER_PROPERTIES.to_string());
        metadata.insert(USER_PROPERTIES_KEY.to_string(), value);

        let trigger = ServiceBusTrigger::new(data, &mut metadata);

        assert_eq!(trigger.delivery_count, DELIVERY_COUNT);
        assert_eq!(trigger.dead_letter_source.unwrap(), DEAD_LETTER_SOURCE);
        assert_eq!(trigger.expiration_time.to_rfc3339(), now.to_rfc3339());
        assert_eq!(trigger.enqueued_time.to_rfc3339(), now.to_rfc3339());
        assert_eq!(trigger.message_id, MESSAGE_ID);
        assert_eq!(trigger.content_type.unwrap(), CONTENT_TYPE);
        assert_eq!(trigger.reply_to.unwrap(), REPLY_TO);
        assert_eq!(trigger.sequence_number, SEQUENCE_NUMBER);
        assert_eq!(trigger.to.unwrap(), TO);
        assert_eq!(trigger.label.unwrap(), LABEL);
        assert_eq!(trigger.correlation_id.unwrap(), CORRELATION_ID);
        assert_eq!(trigger.user_properties.len(), 1);
        assert_eq!(trigger.user_properties["hello"].as_str().unwrap(), "world");
        assert_eq!(trigger.message.as_str().unwrap(), MESSAGE);
    }
}
