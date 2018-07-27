use queue::MessageBody;
use rpc::protocol;

/// Represents an Azure Storage Queue message output binding.
#[derive(Debug)]
pub struct QueueMessage(protocol::TypedData);

impl<T> From<T> for QueueMessage
where
    T: Into<MessageBody<'_>>,
{
    fn from(data: T) -> Self {
        let data: MessageBody = data.into();
        QueueMessage(data.into())
    }
}

impl Into<protocol::TypedData> for QueueMessage {
    fn into(self) -> protocol::TypedData {
        self.0
    }
}
