use queue::MessageBody;
use rpc::protocol;

/// Represents an Azure Storage Queue message output binding.
#[derive(Debug)]
pub struct QueueMessage(protocol::TypedData);

impl<'a, T> From<T> for QueueMessage
where
    T: Into<MessageBody<'a>>,
{
    fn from(data: T) -> Self {
        let data: MessageBody = data.into();
        QueueMessage(data.into())
    }
}

impl<'a> From<&'a MessageBody<'a>> for QueueMessage {
    fn from(data: &'a MessageBody) -> Self {
        QueueMessage(data.clone().into())
    }
}

impl<'a> Into<protocol::TypedData> for QueueMessage {
    fn into(self) -> protocol::TypedData {
        self.0
    }
}
