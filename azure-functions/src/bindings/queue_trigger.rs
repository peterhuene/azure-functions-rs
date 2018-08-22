use bindings::{QueueMessage, Trigger};
use chrono::{DateTime, Utc};
use rpc::protocol;
use std::collections::HashMap;
use util::convert_from;

const ID_KEY: &'static str = "Id";
const DEQUEUE_COUNT_KEY: &'static str = "DequeueCount";
const EXPIRATION_TIME_KEY: &'static str = "ExpirationTime";
const INSERTION_TIME_KEY: &'static str = "InsertionTime";
const NEXT_VISIBLE_TIME_KEY: &'static str = "NextVisibleTime";
const POP_RECEIPT_KEY: &'static str = "PopReceipt";

/// Represents a queue trigger binding.
///
/// # Examples
///
/// A function that runs when a message is posted to a queue called `example`:
///
/// ```rust
/// # extern crate azure_functions;
/// # #[macro_use] extern crate log;
/// use azure_functions::bindings::QueueTrigger;
/// use azure_functions::func;
///
/// #[func]
/// #[binding(name = "trigger", queue_name = "example")]
/// pub fn run_on_message(trigger: &QueueTrigger) {
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
    pub expiration_time: Option<DateTime<Utc>>,
    /// The time that the message was added to the queue.
    pub insertion_time: Option<DateTime<Utc>>,
    /// The time that the message will next be visible.
    pub next_visible_time: Option<DateTime<Utc>>,
    /// The message's pop receipt.
    pub pop_receipt: String,
}

impl From<protocol::TypedData> for QueueTrigger {
    fn from(data: protocol::TypedData) -> Self {
        QueueTrigger {
            message: data.into(),
            id: String::new(),
            dequeue_count: 1,
            expiration_time: None,
            insertion_time: None,
            next_visible_time: None,
            pop_receipt: String::new(),
        }
    }
}

impl Trigger for QueueTrigger {
    fn read_metadata(&mut self, metadata: &mut HashMap<String, protocol::TypedData>) {
        if let Some(id) = metadata.get_mut(ID_KEY) {
            self.id = id.take_string();
        }
        if let Some(count) = metadata.get(DEQUEUE_COUNT_KEY) {
            self.dequeue_count = convert_from(count).expect(&format!(
                "failed to read '{}' from metadata",
                DEQUEUE_COUNT_KEY
            ));
        }
        if let Some(time) = metadata.get(EXPIRATION_TIME_KEY) {
            self.expiration_time = convert_from(time).map(|t| Some(t)).expect(&format!(
                "failed to read '{}' from metadata",
                EXPIRATION_TIME_KEY
            ));
        }
        if let Some(time) = metadata.get(INSERTION_TIME_KEY) {
            self.insertion_time = convert_from(time).map(|t| Some(t)).expect(&format!(
                "failed to read '{}' from metadata",
                INSERTION_TIME_KEY
            ));
        }
        if let Some(time) = metadata.get(NEXT_VISIBLE_TIME_KEY) {
            self.next_visible_time = convert_from(time).map(|t| Some(t)).expect(&format!(
                "failed to read '{}' from metadata",
                NEXT_VISIBLE_TIME_KEY
            ));
        }
        if let Some(receipt) = metadata.get_mut(POP_RECEIPT_KEY) {
            self.pop_receipt = receipt.take_string();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_converts_from_typed_data() {
        const MESSAGE: &'static str = "hello world!";

        let mut data = protocol::TypedData::new();
        data.set_string(MESSAGE.to_string());

        let trigger: QueueTrigger = data.into();
        assert_eq!(trigger.id, "");
        assert_eq!(trigger.dequeue_count, 1);
        assert!(trigger.expiration_time.is_none());
        assert!(trigger.insertion_time.is_none());
        assert!(trigger.next_visible_time.is_none());
        assert_eq!(trigger.pop_receipt, "");
        assert_eq!(trigger.message.as_str().unwrap(), MESSAGE);
    }

    #[test]
    fn it_reads_metadata() {
        const ID: &'static str = "12345";
        const DEQUEUE_COUNT: u32 = 101;
        const POP_RECEIPT: &'static str = "pop!";
        const MESSAGE: &'static str = "\"hello world\"";
        let now = Utc::now();

        let mut data = protocol::TypedData::new();
        data.set_json(MESSAGE.to_string());

        let mut metadata = HashMap::new();

        let mut value = protocol::TypedData::new();
        value.set_string(ID.to_string());
        metadata.insert(ID_KEY.to_string(), value);

        let mut value = protocol::TypedData::new();
        value.set_json(DEQUEUE_COUNT.to_string());
        metadata.insert(DEQUEUE_COUNT_KEY.to_string(), value);

        let mut value = protocol::TypedData::new();
        value.set_string(now.to_rfc3339());
        metadata.insert(EXPIRATION_TIME_KEY.to_string(), value);

        let mut value = protocol::TypedData::new();
        value.set_string(now.to_rfc3339());
        metadata.insert(INSERTION_TIME_KEY.to_string(), value);

        let mut value = protocol::TypedData::new();
        value.set_json("\"".to_string() + &now.to_rfc3339() + "\"");
        metadata.insert(NEXT_VISIBLE_TIME_KEY.to_string(), value);

        let mut value = protocol::TypedData::new();
        value.set_string(POP_RECEIPT.to_string());
        metadata.insert(POP_RECEIPT_KEY.to_string(), value);

        let mut trigger: QueueTrigger = data.into();
        trigger.read_metadata(&mut metadata);
        assert_eq!(trigger.id, ID);
        assert_eq!(trigger.dequeue_count, DEQUEUE_COUNT);
        assert_eq!(
            trigger.expiration_time.unwrap().to_rfc3339(),
            now.to_rfc3339()
        );
        assert_eq!(
            trigger.insertion_time.unwrap().to_rfc3339(),
            now.to_rfc3339()
        );
        assert_eq!(
            trigger.next_visible_time.unwrap().to_rfc3339(),
            now.to_rfc3339()
        );
        assert_eq!(trigger.pop_receipt, POP_RECEIPT);
        assert_eq!(trigger.message.as_str().unwrap(), MESSAGE);
    }
}
