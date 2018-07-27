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
    /// Represents a byte message.
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
