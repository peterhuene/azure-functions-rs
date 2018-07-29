use rpc::protocol;
use serde::de::Error;
use serde::Deserialize;
use serde_json::{from_str, Result, Value};
use std::borrow::Cow;
use std::fmt;
use std::str::from_utf8;

/// Represents an Azure Storage Queue message body.
#[derive(Clone, Debug)]
pub enum MessageBody<'a> {
    /// Represents a string message.
    String(Cow<'a, str>),
    /// Represents a JSON message.
    Json(Cow<'a, str>),
    /// Represents a message of bytes.
    Bytes(Cow<'a, [u8]>),
}

impl MessageBody<'_> {
    /// Gets the contents of the message as a string.
    ///
    /// Returns None if there is no valid string representation of the message.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            MessageBody::String(s) => Some(s),
            MessageBody::Json(s) => Some(s),
            MessageBody::Bytes(b) => from_utf8(b).map(|s| s).ok(),
        }
    }

    /// Gets the contents of the message as a slice of bytes.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            MessageBody::String(s) => s.as_bytes(),
            MessageBody::Json(s) => s.as_bytes(),
            MessageBody::Bytes(b) => b,
        }
    }

    /// Deserializes the message as JSON to the requested type.
    pub fn from_json<T>(&'b self) -> Result<T>
    where
        T: Deserialize<'b>,
    {
        match self {
            MessageBody::String(s) => from_str(s.as_ref()),
            MessageBody::Json(s) => from_str(s.as_ref()),
            MessageBody::Bytes(b) => from_str(from_utf8(b).map_err(|e| {
                ::serde_json::Error::custom(format!("message is not valid UTF-8: {}", e))
            })?),
        }
    }
}

impl fmt::Display for MessageBody<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str().unwrap_or(""))
    }
}

impl From<&'a protocol::TypedData> for MessageBody<'a> {
    fn from(data: &'a protocol::TypedData) -> Self {
        if data.has_string() {
            return MessageBody::String(Cow::Borrowed(data.get_string()));
        }
        if data.has_json() {
            return MessageBody::Json(Cow::Borrowed(data.get_json()));
        }
        if data.has_bytes() {
            return MessageBody::Bytes(Cow::Borrowed(data.get_bytes()));
        }
        if data.has_stream() {
            return MessageBody::Bytes(Cow::Borrowed(data.get_stream()));
        }

        panic!("unexpected data for queue message");
    }
}

impl From<&'a str> for MessageBody<'a> {
    fn from(data: &'a str) -> Self {
        MessageBody::String(Cow::Borrowed(data))
    }
}

impl From<String> for MessageBody<'_> {
    fn from(data: String) -> Self {
        MessageBody::String(Cow::Owned(data))
    }
}

impl From<Value> for MessageBody<'_> {
    fn from(data: Value) -> Self {
        MessageBody::Json(Cow::Owned(data.to_string()))
    }
}

impl From<&'a [u8]> for MessageBody<'a> {
    fn from(data: &'a [u8]) -> Self {
        MessageBody::Bytes(Cow::Borrowed(data))
    }
}

impl From<Vec<u8>> for MessageBody<'_> {
    fn from(data: Vec<u8>) -> Self {
        MessageBody::Bytes(Cow::Owned(data))
    }
}

impl Into<protocol::TypedData> for MessageBody<'_> {
    fn into(self) -> protocol::TypedData {
        let mut data = protocol::TypedData::new();

        match self {
            MessageBody::String(s) => data.set_string(s.into_owned()),
            MessageBody::Json(s) => data.set_json(s.into_owned()),
            MessageBody::Bytes(b) => data.set_bytes(b.into_owned()),
        };

        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_value;
    use std::fmt::Write;

    #[test]
    fn it_has_a_string_body() {
        const BODY: &'static str = "test body";

        let body: MessageBody = BODY.into();
        assert_eq!(body.as_str().unwrap(), BODY);

        let data: protocol::TypedData = body.into();
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

        let body: MessageBody = ::serde_json::to_value(data).unwrap().into();
        assert_eq!(body.from_json::<Data>().unwrap().message, MESSAGE);

        let data: protocol::TypedData = body.into();
        assert_eq!(data.get_json(), r#"{"message":"test"}"#);
    }

    #[test]
    fn it_has_a_bytes_body() {
        const BODY: &'static [u8] = &[1, 2, 3];

        let body: MessageBody = BODY.into();
        assert_eq!(body.as_bytes(), BODY);

        let data: protocol::TypedData = body.into();
        assert_eq!(data.get_bytes(), BODY);
    }

    #[test]
    fn it_displays_as_a_string() {
        const BODY: &'static str = "test";

        let body: MessageBody = BODY.into();

        let mut s = String::new();
        write!(s, "{}", body);

        assert_eq!(s, BODY);
    }

    #[test]
    fn it_converts_from_typed_data() {
        let mut data = protocol::TypedData::new();
        data.set_string("test".to_string());
        let body: MessageBody = (&data).into();
        assert!(matches!(body, MessageBody::String(_)));
        assert_eq!(body.as_str().unwrap(), "test");

        let mut data = protocol::TypedData::new();
        data.set_json("test".to_string());
        let body: MessageBody = (&data).into();
        assert!(matches!(body, MessageBody::Json(_)));
        assert_eq!(body.as_str().unwrap(), "test");

        let mut data = protocol::TypedData::new();
        data.set_bytes(vec![0, 1, 2]);
        let body: MessageBody = (&data).into();
        assert!(matches!(body, MessageBody::Bytes(_)));
        assert_eq!(body.as_bytes(), [0, 1, 2]);

        let mut data = protocol::TypedData::new();
        data.set_stream(vec![0, 1, 2]);
        let body: MessageBody = (&data).into();
        assert!(matches!(body, MessageBody::Bytes(_)));
        assert_eq!(body.as_bytes(), [0, 1, 2]);
    }

    #[test]
    fn it_converts_from_str() {
        let body: MessageBody = "test".into();
        assert!(matches!(body, MessageBody::String(Cow::Borrowed(_))));
        assert_eq!(body.as_str().unwrap(), "test");
    }

    #[test]
    fn it_converts_from_string() {
        let body: MessageBody = "test".to_string().into();
        assert!(matches!(body, MessageBody::String(Cow::Owned(_))));
        assert_eq!(body.as_str().unwrap(), "test");
    }

    #[test]
    fn it_converts_from_json() {
        let body: MessageBody = to_value("hello world").unwrap().into();
        assert!(matches!(body, MessageBody::Json(Cow::Owned(_))));
        assert_eq!(body.as_str().unwrap(), r#""hello world""#);
    }

    #[test]
    fn it_converts_from_u8_slice() {
        let body: MessageBody = [0, 1, 2][..].into();
        assert!(matches!(body, MessageBody::Bytes(Cow::Borrowed(_))));
        assert_eq!(body.as_bytes(), [0, 1, 2]);
    }

    #[test]
    fn it_converts_from_u8_vec() {
        let body: MessageBody = vec![0, 1, 2].into();
        assert!(matches!(body, MessageBody::Bytes(Cow::Owned(_))));
        assert_eq!(body.as_bytes(), [0, 1, 2]);
    }

    #[test]
    fn it_converts_to_typed_data() {
        let body: MessageBody = "test".into();
        let data: protocol::TypedData = body.into();
        assert!(data.has_string());
        assert_eq!(data.get_string(), "test");

        let body: MessageBody = to_value("test").unwrap().into();
        let data: protocol::TypedData = body.into();
        assert!(data.has_json());
        assert_eq!(data.get_json(), r#""test""#);

        let body: MessageBody = vec![1, 2, 3].into();
        let data: protocol::TypedData = body.into();
        assert!(data.has_bytes());
        assert_eq!(data.get_bytes(), [1, 2, 3]);
    }
}
