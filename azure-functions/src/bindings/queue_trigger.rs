use bindings::Trigger;
use chrono::{DateTime, Utc};
use queue::MessageBody;
use rpc::protocol;
use std::collections::HashMap;
use util::convert_from;

/// Represents a queue trigger binding.
#[derive(Debug)]
pub struct QueueTrigger<'a> {
    data: &'a protocol::TypedData,
    /// The queue message identifier.
    pub id: &'a str,
    /// The number of times this message has been dequeued.
    pub dequeue_count: u32,
    /// The time that the message expires.
    pub expiration_time: Option<DateTime<Utc>>,
    /// The time that the message was added to the queue.
    pub insertion_time: Option<DateTime<Utc>>,
    /// The time that the message will next be visible.
    pub next_visible_time: Option<DateTime<Utc>>,
    /// The message's pop receipt.
    pub pop_receipt: &'a str,
}

impl<'a> QueueTrigger<'a> {
    /// Gets the message that triggered the function.
    pub fn message(&self) -> MessageBody {
        MessageBody::from(self.data)
    }
}

impl<'a> From<&'a protocol::TypedData> for QueueTrigger<'a> {
    fn from(data: &'a protocol::TypedData) -> Self {
        QueueTrigger {
            data: data,
            id: "",
            dequeue_count: 1,
            expiration_time: None,
            insertion_time: None,
            next_visible_time: None,
            pop_receipt: "",
        }
    }
}

impl<'a> Trigger<'a> for QueueTrigger<'a> {
    fn read_metadata(&mut self, metadata: &'a HashMap<String, protocol::TypedData>) {
        if let Some(id) = metadata.get("Id") {
            self.id = id.get_string();
        }
        if let Some(count) = metadata.get("DequeueCount") {
            self.dequeue_count =
                convert_from(count).expect("failed to read 'DequeueCount' from metadata");
        }
        if let Some(time) = metadata.get("ExpirationTime") {
            self.expiration_time = convert_from(time)
                .map(|t| Some(t))
                .expect("failed to read 'ExpirationTime' from metadata");
        }
        if let Some(time) = metadata.get("InsertionTime") {
            self.insertion_time = convert_from(time)
                .map(|t| Some(t))
                .expect("failed to read 'InsertionTime' from metadata");
        }
        if let Some(time) = metadata.get("NextVisibleTime") {
            self.next_visible_time = convert_from(time)
                .map(|t| Some(t))
                .expect("failed to read 'NextVisibleTime' from metadata");
        }
        if let Some(receipt) = metadata.get("PopReceipt") {
            self.pop_receipt = receipt.get_string();
        }
    }
}
