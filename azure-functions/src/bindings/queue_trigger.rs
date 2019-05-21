use crate::{
    bindings::QueueMessage,
    rpc::{typed_data::Data, TypedData},
    util::convert_from,
};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

const ID_KEY: &str = "Id";
const DEQUEUE_COUNT_KEY: &str = "DequeueCount";
const EXPIRATION_TIME_KEY: &str = "ExpirationTime";
const INSERTION_TIME_KEY: &str = "InsertionTime";
const NEXT_VISIBLE_TIME_KEY: &str = "NextVisibleTime";
const POP_RECEIPT_KEY: &str = "PopReceipt";

/// Represents a queue trigger binding.
///
/// The following binding attributes are supported:
///
/// | Name         | Description                                                                                                                                  |
/// |--------------|----------------------------------------------------------------------------------------------------------------------------------------------|
/// | `name`       | The name of the parameter being bound.                                                                                                       |
/// | `queue_name` | The name of the queue to poll.                                                                                                               |
/// | `connection` | The name of an app setting that contains the Azure Storage connection string to use for this binding. Defaults to the `AzureWebJobsStorage`. |
///
/// # Examples
///
/// A function that runs when a message is posted to a queue called `example`:
///
/// ```rust
/// use azure_functions::bindings::QueueTrigger;
/// use azure_functions::func;
/// use log::info;
///
/// #[func]
/// #[binding(name = "trigger", queue_name = "example")]
/// pub fn run_on_message(trigger: QueueTrigger) {
///     info!("Rust function ran due to queue message: {}", trigger.message);
/// }
/// ```
#[derive(Debug)]
pub struct QueueTrigger {
    /// The queue message that triggered the function.
    pub message: QueueMessage,
    /// The queue message identifier.
    pub id: String,
    /// The number of times this message has been dequeued.
    pub dequeue_count: u32,
    /// The time that the message expires.
    pub expiration_time: DateTime<Utc>,
    /// The time that the message was added to the queue.
    pub insertion_time: DateTime<Utc>,
    /// The time that the message will next be visible.
    pub next_visible_time: DateTime<Utc>,
    /// The message's pop receipt.
    pub pop_receipt: String,
}

impl QueueTrigger {
    #[doc(hidden)]
    pub fn new(data: TypedData, mut metadata: HashMap<String, TypedData>) -> Self {
        QueueTrigger {
            message: data.into(),
            id: metadata
                .remove(ID_KEY)
                .map(|data| match data.data {
                    Some(Data::String(s)) => s,
                    _ => panic!("expected a string for message id"),
                })
                .expect("expected a message id"),
            dequeue_count: convert_from(
                metadata
                    .get(DEQUEUE_COUNT_KEY)
                    .expect("expected a dequeue count"),
            )
            .expect("failed to convert dequeue count"),
            expiration_time: convert_from(
                metadata
                    .get(EXPIRATION_TIME_KEY)
                    .expect("expected an expiration time"),
            )
            .expect("failed to convert expiration time"),
            insertion_time: convert_from(
                metadata
                    .get(INSERTION_TIME_KEY)
                    .expect("expected an insertion time"),
            )
            .expect("failed to convert insertion time"),
            next_visible_time: convert_from(
                metadata
                    .get(NEXT_VISIBLE_TIME_KEY)
                    .expect("expected a next visible time"),
            )
            .expect("failed to convert next visible time"),
            pop_receipt: metadata
                .remove(POP_RECEIPT_KEY)
                .map(|data| match data.data {
                    Some(Data::String(s)) => s,
                    _ => panic!("expected a string for pop receipt"),
                })
                .expect("expected a pop receipt"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_constructs() {
        const ID: &'static str = "12345";
        const DEQUEUE_COUNT: u32 = 101;
        const POP_RECEIPT: &'static str = "pop!";
        const MESSAGE: &'static str = "\"hello world\"";
        let now = Utc::now();

        let data = TypedData {
            data: Some(Data::Json(MESSAGE.to_string())),
        };

        let mut metadata = HashMap::new();

        metadata.insert(
            ID_KEY.to_string(),
            TypedData {
                data: Some(Data::String(ID.to_string())),
            },
        );

        metadata.insert(
            DEQUEUE_COUNT_KEY.to_string(),
            TypedData {
                data: Some(Data::Json(DEQUEUE_COUNT.to_string())),
            },
        );

        metadata.insert(
            EXPIRATION_TIME_KEY.to_string(),
            TypedData {
                data: Some(Data::String(now.to_rfc3339())),
            },
        );

        metadata.insert(
            INSERTION_TIME_KEY.to_string(),
            TypedData {
                data: Some(Data::String(now.to_rfc3339())),
            },
        );

        metadata.insert(
            NEXT_VISIBLE_TIME_KEY.to_string(),
            TypedData {
                data: Some(Data::Json("\"".to_string() + &now.to_rfc3339() + "\"")),
            },
        );

        metadata.insert(
            POP_RECEIPT_KEY.to_string(),
            TypedData {
                data: Some(Data::String(POP_RECEIPT.to_string())),
            },
        );

        let trigger = QueueTrigger::new(data, metadata);
        assert_eq!(trigger.id, ID);
        assert_eq!(trigger.dequeue_count, DEQUEUE_COUNT);
        assert_eq!(trigger.expiration_time.to_rfc3339(), now.to_rfc3339());
        assert_eq!(trigger.insertion_time.to_rfc3339(), now.to_rfc3339());
        assert_eq!(trigger.next_visible_time.to_rfc3339(), now.to_rfc3339());
        assert_eq!(trigger.pop_receipt, POP_RECEIPT);
        assert_eq!(trigger.message.as_str().unwrap(), MESSAGE);
    }
}
