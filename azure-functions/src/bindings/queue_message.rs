use queue::MessageBody;
use rpc::protocol;

/// Represents an Azure Storage Queue message output binding.
///
/// Queue messages can be created for any type that implements `Into<MessageBody>`.
///
/// # Examples
///
/// Creating a queue message from a string:
///
/// ```rust
/// use azure_functions::bindings::QueueMessage;
///
/// let message: QueueMessage = "hello world!".into();
/// assert_eq!(message.body().as_str().unwrap(), "hello world!");
/// ```
///
/// Creating a queue message from a JSON value (see the [json! macro](https://docs.serde.rs/serde_json/macro.json.html) from the `serde_json` crate):
///
/// ```rust
/// # #[macro_use] extern crate serde_json;
/// # extern crate azure_functions;
/// use azure_functions::bindings::QueueMessage;
///
/// let message: QueueMessage = json!({ "hello": "world!" }).into();
///
/// assert_eq!(message.body().as_str().unwrap(), r#"{"hello":"world!"}"#);
/// ```
///
/// Creating a queue message from a sequence of bytes:
///
/// ```rust
/// use azure_functions::bindings::QueueMessage;
///
/// let message: QueueMessage = [1, 2, 3][..].into();
///
/// assert_eq!(
///     message.body().as_bytes(),
///     [1, 2, 3]
/// );
/// ```
#[derive(Debug)]
pub struct QueueMessage(protocol::TypedData);

impl QueueMessage {
    /// Gets the body of the message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use azure_functions::bindings::QueueMessage;
    ///
    /// let message: QueueMessage = "hello world!".into();
    /// assert_eq!(message.body().as_str().unwrap(), "hello world!");
    /// ```
    pub fn body(&self) -> MessageBody {
        (&self.0).into()
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_has_a_string_body() {
        const BODY: &'static str = "test body";

        let message: QueueMessage = BODY.into();
        assert_eq!(message.body().as_str().unwrap(), BODY);

        let data: protocol::TypedData = message.into();
        assert_eq!(data.get_string(), BODY);
    }

    #[test]
    fn it_has_a_json_body() {
        #[derive(Serialize, Deserialize)]
        struct Data {
            message: String,
        };

        const MESSAGE: &'static str = "test";

        let data = Data {
            message: MESSAGE.to_string(),
        };

        let message: QueueMessage = ::serde_json::to_value(data).unwrap().into();
        assert_eq!(message.body().from_json::<Data>().unwrap().message, MESSAGE);

        let data: protocol::TypedData = message.into();
        assert_eq!(data.get_json(), r#"{"message":"test"}"#);
    }

    #[test]
    fn it_has_a_bytes_body() {
        const BODY: &'static [u8] = &[1, 2, 3];

        let message: QueueMessage = BODY.into();
        assert_eq!(message.body().as_bytes(), BODY);

        let data: protocol::TypedData = message.into();
        assert_eq!(data.get_bytes(), BODY);
    }
}
