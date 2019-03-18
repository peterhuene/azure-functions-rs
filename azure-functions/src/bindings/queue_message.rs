use crate::{http::Body, rpc::protocol, FromVec};
use serde::de::Error;
use serde::Deserialize;
use serde_json::{from_str, Result, Value};
use std::borrow::Cow;
use std::fmt;
use std::str::from_utf8;

/// Represents an Azure Storage Queue message output binding.
///
/// # Examples
///
/// Creating a queue message from a string:
///
/// ```rust
/// use azure_functions::bindings::{HttpRequest, QueueMessage};
/// use azure_functions::func;
///
/// #[func]
/// #[binding(name = "output1", queue_name = "example")]
/// pub fn example(_req: HttpRequest) -> ((), QueueMessage) {
///     ((), "Hello world!".into())
/// }
/// ```
///
/// Creating a queue message from a JSON value (see the [json! macro](https://docs.serde.rs/serde_json/macro.json.html) from the `serde_json` crate):
///
/// ```rust
/// use azure_functions::bindings::{HttpRequest, QueueMessage};
/// use azure_functions::func;
/// use serde_json::json;
///
/// #[func]
/// #[binding(name = "output1", queue_name = "example")]
/// pub fn example(_req: HttpRequest) -> ((), QueueMessage) {
///     ((), json!({ "hello": "world" }).into())
/// }
/// ```
///
/// Creating a queue message from a sequence of bytes:
///
/// ```rust
/// use azure_functions::bindings::{HttpRequest, QueueMessage};
/// use azure_functions::func;
/// use serde_json::json;
///
/// #[func]
/// #[binding(name = "output1", queue_name = "example")]
/// pub fn example(_req: HttpRequest) -> ((), QueueMessage) {
///     ((), [1, 2, 3][..].into())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct QueueMessage(protocol::TypedData);

impl QueueMessage {
    /// Gets the content of the message as a string.
    ///
    /// Returns None if there is no valid string representation of the message.
    pub fn as_str(&self) -> Option<&str> {
        if self.0.has_string() {
            return Some(self.0.get_string());
        }
        if self.0.has_json() {
            return Some(self.0.get_json());
        }
        if self.0.has_bytes() {
            return from_utf8(self.0.get_bytes()).map(|s| s).ok();
        }
        if self.0.has_stream() {
            return from_utf8(self.0.get_stream()).map(|s| s).ok();
        }
        None
    }

    /// Gets the content of the message as a slice of bytes.
    pub fn as_bytes(&self) -> &[u8] {
        if self.0.has_string() {
            return self.0.get_string().as_bytes();
        }
        if self.0.has_json() {
            return self.0.get_json().as_bytes();
        }
        if self.0.has_bytes() {
            return self.0.get_bytes();
        }
        if self.0.has_stream() {
            return self.0.get_stream();
        }

        panic!("unexpected data for queue message contents");
    }

    /// Deserializes the message as JSON to the requested type.
    pub fn as_json<'b, T>(&'b self) -> Result<T>
    where
        T: Deserialize<'b>,
    {
        from_str(
            self.as_str()
                .ok_or_else(|| ::serde_json::Error::custom("queue message is not valid UTF-8"))?,
        )
    }
}

impl fmt::Display for QueueMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str().unwrap_or(""))
    }
}

impl<'a> From<&'a str> for QueueMessage {
    fn from(content: &'a str) -> Self {
        let mut data = protocol::TypedData::new();
        data.set_string(content.to_owned());
        QueueMessage(data)
    }
}

impl From<String> for QueueMessage {
    fn from(content: String) -> Self {
        let mut data = protocol::TypedData::new();
        data.set_string(content);
        QueueMessage(data)
    }
}

impl From<&Value> for QueueMessage {
    fn from(content: &Value) -> Self {
        let mut data = protocol::TypedData::new();
        data.set_json(content.to_string());
        QueueMessage(data)
    }
}

impl From<Value> for QueueMessage {
    fn from(content: Value) -> Self {
        let mut data = protocol::TypedData::new();
        data.set_json(content.to_string());
        QueueMessage(data)
    }
}

impl<'a> From<&'a [u8]> for QueueMessage {
    fn from(content: &'a [u8]) -> Self {
        let mut data = protocol::TypedData::new();
        data.set_bytes(content.to_owned());
        QueueMessage(data)
    }
}

impl From<Vec<u8>> for QueueMessage {
    fn from(content: Vec<u8>) -> Self {
        let mut data = protocol::TypedData::new();
        data.set_bytes(content);
        QueueMessage(data)
    }
}

#[doc(hidden)]
impl From<protocol::TypedData> for QueueMessage {
    fn from(data: protocol::TypedData) -> Self {
        QueueMessage(data)
    }
}

#[doc(hidden)]
impl FromVec<QueueMessage> for protocol::TypedData {
    fn from_vec(vec: Vec<QueueMessage>) -> Self {
        let mut data = protocol::TypedData::new();
        data.set_json(Value::Array(vec.into_iter().map(Into::into).collect()).to_string());
        data
    }
}

impl Into<String> for QueueMessage {
    fn into(mut self) -> String {
        if self.0.has_string() {
            return self.0.take_string();
        }
        if self.0.has_json() {
            return self.0.take_json();
        }
        if self.0.has_bytes() {
            return String::from_utf8(self.0.take_bytes())
                .expect("queue message does not contain valid UTF-8 bytes");
        }
        if self.0.has_stream() {
            return String::from_utf8(self.0.take_stream())
                .expect("queue message does not contain valid UTF-8 bytes");
        }
        panic!("unexpected data for queue message content");
    }
}

impl Into<Value> for QueueMessage {
    fn into(mut self) -> Value {
        if self.0.has_string() {
            return Value::String(self.0.take_string());
        }
        if self.0.has_json() {
            return from_str(self.0.get_json())
                .expect("queue message does not contain valid JSON data");
        }
        // TODO: this is not an efficient encoding
        if self.0.has_bytes() {
            return Value::Array(
                self.0
                    .get_bytes()
                    .iter()
                    .map(|n| Value::Number(u64::from(*n).into()))
                    .collect(),
            );
        }
        // TODO: this is not an efficient encoding
        if self.0.has_stream() {
            return Value::Array(
                self.0
                    .get_stream()
                    .iter()
                    .map(|n| Value::Number(u64::from(*n).into()))
                    .collect(),
            );
        }
        panic!("unexpected data for queue message content");
    }
}

impl Into<Vec<u8>> for QueueMessage {
    fn into(mut self) -> Vec<u8> {
        if self.0.has_string() {
            return self.0.take_string().into_bytes();
        }
        if self.0.has_json() {
            return self.0.take_json().into_bytes();
        }
        if self.0.has_bytes() {
            return self.0.take_bytes();
        }
        if self.0.has_stream() {
            return self.0.take_stream();
        }

        panic!("unexpected data for queue message content");
    }
}

impl<'a> Into<Body<'a>> for QueueMessage {
    fn into(mut self) -> Body<'a> {
        if self.0.has_string() {
            return self.0.take_string().into();
        }
        if self.0.has_json() {
            return Body::Json(Cow::from(self.0.take_json()));
        }
        if self.0.has_bytes() {
            return self.0.take_bytes().into();
        }
        if self.0.has_stream() {
            return self.0.take_stream().into();
        }

        panic!("unexpected data for blob content");
    }
}

#[doc(hidden)]
impl Into<protocol::TypedData> for QueueMessage {
    fn into(self) -> protocol::TypedData {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_derive::{Deserialize, Serialize};
    use serde_json::{json, to_value};
    use std::fmt::Write;

    #[test]
    fn it_has_string_content() {
        const MESSAGE: &'static str = "test message";

        let message: QueueMessage = MESSAGE.into();
        assert_eq!(message.as_str().unwrap(), MESSAGE);

        let data: protocol::TypedData = message.into();
        assert_eq!(data.get_string(), MESSAGE);
    }

    #[test]
    fn it_has_json_content() {
        #[derive(Serialize, Deserialize)]
        struct Data {
            message: String,
        };

        const MESSAGE: &'static str = "test";

        let data = Data {
            message: MESSAGE.to_string(),
        };

        let message: QueueMessage = ::serde_json::to_value(data).unwrap().into();
        assert_eq!(message.as_json::<Data>().unwrap().message, MESSAGE);

        let data: protocol::TypedData = message.into();
        assert_eq!(data.get_json(), r#"{"message":"test"}"#);
    }

    #[test]
    fn it_has_bytes_content() {
        const MESSAGE: &'static [u8] = &[1, 2, 3];

        let message: QueueMessage = MESSAGE.into();
        assert_eq!(message.as_bytes(), MESSAGE);

        let data: protocol::TypedData = message.into();
        assert_eq!(data.get_bytes(), MESSAGE);
    }

    #[test]
    fn it_displays_as_a_string() {
        const MESSAGE: &'static str = "test";

        let message: QueueMessage = MESSAGE.into();

        let mut s = String::new();
        write!(s, "{}", message).unwrap();

        assert_eq!(s, MESSAGE);
    }

    #[test]
    fn it_converts_from_str() {
        let message: QueueMessage = "test".into();
        assert_eq!(message.as_str().unwrap(), "test");
    }

    #[test]
    fn it_converts_from_string() {
        let message: QueueMessage = "test".to_string().into();
        assert_eq!(message.as_str().unwrap(), "test");
    }

    #[test]
    fn it_converts_from_json() {
        let message: QueueMessage = to_value("hello world").unwrap().into();
        assert_eq!(message.as_str().unwrap(), r#""hello world""#);
    }

    #[test]
    fn it_converts_from_u8_slice() {
        let message: QueueMessage = [0, 1, 2][..].into();
        assert_eq!(message.as_bytes(), [0, 1, 2]);
    }

    #[test]
    fn it_converts_from_u8_vec() {
        let message: QueueMessage = vec![0, 1, 2].into();
        assert_eq!(message.as_bytes(), [0, 1, 2]);
    }

    #[test]
    fn it_converts_to_string() {
        let message: QueueMessage = "hello world!".into();
        let s: String = message.into();
        assert_eq!(s, "hello world!");
    }

    #[test]
    fn it_converts_to_json() {
        let message: QueueMessage = json!({"hello": "world"}).into();
        let value: Value = message.into();
        assert_eq!(value.to_string(), r#"{"hello":"world"}"#);
    }

    #[test]
    fn it_converts_to_bytes() {
        let message: QueueMessage = vec![1, 2, 3].into();
        let bytes: Vec<u8> = message.into();
        assert_eq!(bytes, [1, 2, 3]);
    }

    #[test]
    fn it_converts_to_body() {
        let message: QueueMessage = "hello world!".into();
        let body: Body = message.into();
        assert_eq!(body.as_str().unwrap(), "hello world!");

        let message: QueueMessage = json!({"hello": "world"}).into();
        let body: Body = message.into();
        assert_eq!(body.as_str().unwrap(), r#"{"hello":"world"}"#);

        let message: QueueMessage = vec![1, 2, 3].into();
        let body: Body = message.into();
        assert_eq!(body.as_bytes(), [1, 2, 3]);
    }

    #[test]
    fn it_converts_to_typed_data() {
        let message: QueueMessage = "test".into();
        let data: protocol::TypedData = message.into();
        assert!(data.has_string());
        assert_eq!(data.get_string(), "test");

        let message: QueueMessage = to_value("test").unwrap().into();
        let data: protocol::TypedData = message.into();
        assert!(data.has_json());
        assert_eq!(data.get_json(), r#""test""#);

        let message: QueueMessage = vec![1, 2, 3].into();
        let data: protocol::TypedData = message.into();
        assert!(data.has_bytes());
        assert_eq!(data.get_bytes(), [1, 2, 3]);
    }
}
