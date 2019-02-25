use crate::rpc::protocol;
use serde::de::Error;
use serde::Deserialize;
use serde_json::{from_str, Result, Value};
use std::fmt;
use std::str::from_utf8;

/// Represents an Event Hubs message output binding.
///
/// # Examples
///
/// Creating a message from a string:
///
/// ```rust
/// use azure_functions::bindings::EventHubMessage;
///
/// let message: EventHubMessage = "hello world!".into();
/// assert_eq!(message.as_str().unwrap(), "hello world!");
/// ```
///
/// Creating a message from a JSON value (see the [json! macro](https://docs.serde.rs/serde_json/macro.json.html) from the `serde_json` crate):
///
/// ```rust
/// # #[macro_use] extern crate serde_json;
/// # extern crate azure_functions;
/// use azure_functions::bindings::EventHubMessage;
///
/// let message: EventHubMessage = json!({ "hello": "world!" }).into();
///
/// assert_eq!(message.as_str().unwrap(), r#"{"hello":"world!"}"#);
/// ```
///
/// Creating a message from a sequence of bytes:
///
/// ```rust
/// use azure_functions::bindings::EventHubMessage;
///
/// let message: EventHubMessage = [1, 2, 3][..].into();
///
/// assert_eq!(
///     message.as_bytes(),
///     [1, 2, 3]
/// );
/// ```
#[derive(Debug, Clone)]
pub struct EventHubMessage(protocol::TypedData);

impl EventHubMessage {
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

        panic!("unexpected data for Event Hub message contents");
    }

    /// Deserializes the message as JSON to the requested type.
    pub fn as_json<'b, T>(&'b self) -> Result<T>
    where
        T: Deserialize<'b>,
    {
        from_str(
            self.as_str().ok_or_else(|| {
                ::serde_json::Error::custom("Event Hub message is not valid UTF-8")
            })?,
        )
    }
}

impl fmt::Display for EventHubMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str().unwrap_or(""))
    }
}

impl<'a> From<&'a str> for EventHubMessage {
    fn from(content: &'a str) -> Self {
        let mut data = protocol::TypedData::new();
        data.set_string(content.to_owned());
        EventHubMessage(data)
    }
}

impl From<String> for EventHubMessage {
    fn from(content: String) -> Self {
        let mut data = protocol::TypedData::new();
        data.set_string(content);
        EventHubMessage(data)
    }
}

impl From<&Value> for EventHubMessage {
    fn from(content: &Value) -> Self {
        let mut data = protocol::TypedData::new();
        data.set_json(content.to_string());
        EventHubMessage(data)
    }
}

impl From<Value> for EventHubMessage {
    fn from(content: Value) -> Self {
        let mut data = protocol::TypedData::new();
        data.set_json(content.to_string());
        EventHubMessage(data)
    }
}

impl<'a> From<&'a [u8]> for EventHubMessage {
    fn from(content: &'a [u8]) -> Self {
        let mut data = protocol::TypedData::new();
        data.set_bytes(content.to_owned());
        EventHubMessage(data)
    }
}

impl From<Vec<u8>> for EventHubMessage {
    fn from(content: Vec<u8>) -> Self {
        let mut data = protocol::TypedData::new();
        data.set_bytes(content);
        EventHubMessage(data)
    }
}

#[doc(hidden)]
impl From<protocol::TypedData> for EventHubMessage {
    fn from(data: protocol::TypedData) -> Self {
        EventHubMessage(data)
    }
}

#[doc(hidden)]
impl Into<protocol::TypedData> for EventHubMessage {
    fn into(self) -> protocol::TypedData {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_value;
    use std::fmt::Write;

    #[test]
    fn it_has_string_content() {
        const MESSAGE: &'static str = "test message";

        let message: EventHubMessage = MESSAGE.into();
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

        let message: EventHubMessage = ::serde_json::to_value(data).unwrap().into();
        assert_eq!(message.as_json::<Data>().unwrap().message, MESSAGE);

        let data: protocol::TypedData = message.into();
        assert_eq!(data.get_json(), r#"{"message":"test"}"#);
    }

    #[test]
    fn it_has_bytes_content() {
        const MESSAGE: &'static [u8] = &[1, 2, 3];

        let message: EventHubMessage = MESSAGE.into();
        assert_eq!(message.as_bytes(), MESSAGE);

        let data: protocol::TypedData = message.into();
        assert_eq!(data.get_bytes(), MESSAGE);
    }

    #[test]
    fn it_displays_as_a_string() {
        const MESSAGE: &'static str = "test";

        let message: EventHubMessage = MESSAGE.into();

        let mut s = String::new();
        write!(s, "{}", message).unwrap();

        assert_eq!(s, MESSAGE);
    }

    #[test]
    fn it_converts_from_str() {
        let message: EventHubMessage = "test".into();
        assert_eq!(message.as_str().unwrap(), "test");
    }

    #[test]
    fn it_converts_from_string() {
        let message: EventHubMessage = "test".to_string().into();
        assert_eq!(message.as_str().unwrap(), "test");
    }

    #[test]
    fn it_converts_from_json() {
        let message: EventHubMessage = to_value("hello world").unwrap().into();
        assert_eq!(message.as_str().unwrap(), r#""hello world""#);
    }

    #[test]
    fn it_converts_from_u8_slice() {
        let message: EventHubMessage = [0, 1, 2][..].into();
        assert_eq!(message.as_bytes(), [0, 1, 2]);
    }

    #[test]
    fn it_converts_from_u8_vec() {
        let message: EventHubMessage = vec![0, 1, 2].into();
        assert_eq!(message.as_bytes(), [0, 1, 2]);
    }

    #[test]
    fn it_converts_to_typed_data() {
        let message: EventHubMessage = "test".into();
        let data: protocol::TypedData = message.into();
        assert!(data.has_string());
        assert_eq!(data.get_string(), "test");

        let message: EventHubMessage = to_value("test").unwrap().into();
        let data: protocol::TypedData = message.into();
        assert!(data.has_json());
        assert_eq!(data.get_json(), r#""test""#);

        let message: EventHubMessage = vec![1, 2, 3].into();
        let data: protocol::TypedData = message.into();
        assert!(data.has_bytes());
        assert_eq!(data.get_bytes(), [1, 2, 3]);
    }
}
