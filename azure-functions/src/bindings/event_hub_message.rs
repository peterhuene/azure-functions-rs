use crate::{
    http::Body,
    rpc::{typed_data::Data, TypedData},
    FromVec,
};
use serde::de::Error;
use serde::Deserialize;
use serde_json::{from_str, Result, Value};
use std::borrow::Cow;
use std::fmt;
use std::str::from_utf8;

/// Represents an Event Hubs message output binding.
///
/// The following binding attributes are supported:
///
/// | Name             | Description                                                                                                                                                                             |
/// |------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
/// | `name`           | The name of the parameter being bound.                                                                                                                                                  |
/// | `event_hub_name` | The name of the event hub. When the event hub name is also present in the connection string, that value overrides this property at runtime.                                             |
/// | `connection`     | The name of an app setting that contains the connection string to the event hub's namespace. This connection string must have send permissions to send the message to the event stream. |
///
/// # Examples
///
/// Creating a message from a string:
///
/// ```rust
/// use azure_functions::bindings::{HttpRequest, EventHubMessage};
/// use azure_functions::func;
///
/// #[func]
/// #[binding(name = "output1", connection = "connection", event_hub_name = "example")]
/// pub fn create_message(_req: HttpRequest) -> ((), EventHubMessage) {
///     ((), "Hello world!".into())
/// }
/// ```
///
/// Creating a message from a JSON value (see the [json! macro](https://docs.serde.rs/serde_json/macro.json.html) from the `serde_json` crate):
///
/// ```rust
/// use azure_functions::bindings::{HttpRequest, EventHubMessage};
/// use azure_functions::func;
/// use serde_json::json;
///
/// #[func]
/// #[binding(name = "output1", connection = "connection", event_hub_name = "example")]
/// pub fn create_message(_req: HttpRequest) -> ((), EventHubMessage) {
///     (() ,json!({ "hello": "world!" }).into())
/// }
/// ```
///
/// Creating a message from a sequence of bytes:
///
/// ```rust
/// use azure_functions::bindings::{HttpRequest, EventHubMessage};
/// use azure_functions::func;
///
/// #[func]
/// #[binding(name = "output1", connection = "connection", event_hub_name = "example")]
/// pub fn create_message(_req: HttpRequest) -> ((), EventHubMessage) {
///     ((), [1, 2, 3][..].into())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct EventHubMessage(TypedData);

impl EventHubMessage {
    /// Gets the content of the message as a string.
    ///
    /// Returns None if there is no valid string representation of the message.
    pub fn as_str(&self) -> Option<&str> {
        match &self.0.data {
            Some(Data::String(s)) => Some(s),
            Some(Data::Json(s)) => Some(s),
            Some(Data::Bytes(b)) => from_utf8(b).ok(),
            Some(Data::Stream(s)) => from_utf8(s).ok(),
            _ => None,
        }
    }

    /// Gets the content of the message as a slice of bytes.
    pub fn as_bytes(&self) -> &[u8] {
        match &self.0.data {
            Some(Data::String(s)) => s.as_bytes(),
            Some(Data::Json(s)) => s.as_bytes(),
            Some(Data::Bytes(b)) => b,
            Some(Data::Stream(s)) => s,
            _ => panic!("unexpected data for Event Hub message contents"),
        }
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
        EventHubMessage(TypedData {
            data: Some(Data::String(content.to_owned())),
        })
    }
}

impl From<String> for EventHubMessage {
    fn from(content: String) -> Self {
        EventHubMessage(TypedData {
            data: Some(Data::String(content)),
        })
    }
}

impl From<&Value> for EventHubMessage {
    fn from(content: &Value) -> Self {
        EventHubMessage(TypedData {
            data: Some(Data::Json(content.to_string())),
        })
    }
}

impl From<Value> for EventHubMessage {
    fn from(content: Value) -> Self {
        EventHubMessage(TypedData {
            data: Some(Data::Json(content.to_string())),
        })
    }
}

impl<'a> From<&'a [u8]> for EventHubMessage {
    fn from(content: &'a [u8]) -> Self {
        EventHubMessage(TypedData {
            data: Some(Data::Bytes(content.to_owned())),
        })
    }
}

impl From<Vec<u8>> for EventHubMessage {
    fn from(content: Vec<u8>) -> Self {
        EventHubMessage(TypedData {
            data: Some(Data::Bytes(content)),
        })
    }
}

#[doc(hidden)]
impl From<TypedData> for EventHubMessage {
    fn from(data: TypedData) -> Self {
        EventHubMessage(data)
    }
}

#[doc(hidden)]
impl FromVec<EventHubMessage> for TypedData {
    fn from_vec(vec: Vec<EventHubMessage>) -> Self {
        TypedData {
            data: Some(Data::Json(
                Value::Array(vec.into_iter().map(Into::into).collect()).to_string(),
            )),
        }
    }
}

impl Into<String> for EventHubMessage {
    fn into(self) -> String {
        match self.0.data {
            Some(Data::String(s)) => s,
            Some(Data::Json(s)) => s,
            Some(Data::Bytes(b)) => {
                String::from_utf8(b).expect("Event Hub message does not contain valid UTF-8 bytes")
            }
            Some(Data::Stream(s)) => {
                String::from_utf8(s).expect("Event Hub message does not contain valid UTF-8 bytes")
            }
            _ => panic!("unexpected data for Event Hub message content"),
        }
    }
}

impl Into<Value> for EventHubMessage {
    fn into(self) -> Value {
        // TODO: this is not an efficient encoding for bytes/stream
        match self.0.data {
            Some(Data::String(s)) => Value::String(s),
            Some(Data::Json(s)) => {
                from_str(&s).expect("Event Hub message does not contain valid JSON data")
            }
            Some(Data::Bytes(b)) => Value::Array(
                b.iter()
                    .map(|n| Value::Number(u64::from(*n).into()))
                    .collect(),
            ),
            Some(Data::Stream(s)) => Value::Array(
                s.iter()
                    .map(|n| Value::Number(u64::from(*n).into()))
                    .collect(),
            ),
            _ => panic!("unexpected data for Event Hub message content"),
        }
    }
}

impl Into<Vec<u8>> for EventHubMessage {
    fn into(self) -> Vec<u8> {
        match self.0.data {
            Some(Data::String(s)) => s.into_bytes(),
            Some(Data::Json(s)) => s.into_bytes(),
            Some(Data::Bytes(b)) => b,
            Some(Data::Stream(s)) => s,
            _ => panic!("unexpected data for Event Hub message content"),
        }
    }
}

impl<'a> Into<Body<'a>> for EventHubMessage {
    fn into(self) -> Body<'a> {
        match self.0.data {
            Some(Data::String(s)) => s.into(),
            Some(Data::Json(s)) => Body::Json(Cow::from(s)),
            Some(Data::Bytes(b)) => b.into(),
            Some(Data::Stream(s)) => s.into(),
            _ => panic!("unexpected data for Event Hub message content"),
        }
    }
}

#[doc(hidden)]
impl Into<TypedData> for EventHubMessage {
    fn into(self) -> TypedData {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use serde_json::{json, to_value};
    use std::fmt::Write;

    #[test]
    fn it_has_string_content() {
        const MESSAGE: &'static str = "test message";

        let message: EventHubMessage = MESSAGE.into();
        assert_eq!(message.as_str().unwrap(), MESSAGE);

        let data: TypedData = message.into();
        assert_eq!(data.data, Some(Data::String(MESSAGE.to_string())));
    }

    #[test]
    fn it_has_json_content() {
        #[derive(Serialize, Deserialize)]
        struct SerializedData {
            message: String,
        };

        const MESSAGE: &'static str = "test";

        let data = SerializedData {
            message: MESSAGE.to_string(),
        };

        let message: EventHubMessage = ::serde_json::to_value(data).unwrap().into();
        assert_eq!(
            message.as_json::<SerializedData>().unwrap().message,
            MESSAGE
        );

        let data: TypedData = message.into();
        assert_eq!(
            data.data,
            Some(Data::Json(r#"{"message":"test"}"#.to_string()))
        );
    }

    #[test]
    fn it_has_bytes_content() {
        const MESSAGE: &'static [u8] = &[1, 2, 3];

        let message: EventHubMessage = MESSAGE.into();
        assert_eq!(message.as_bytes(), MESSAGE);

        let data: TypedData = message.into();
        assert_eq!(data.data, Some(Data::Bytes(MESSAGE.to_owned())));
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
    fn it_converts_to_string() {
        let message: EventHubMessage = "hello world!".into();
        let s: String = message.into();
        assert_eq!(s, "hello world!");
    }

    #[test]
    fn it_converts_to_json() {
        let message: EventHubMessage = json!({"hello": "world"}).into();
        let value: Value = message.into();
        assert_eq!(value.to_string(), r#"{"hello":"world"}"#);
    }

    #[test]
    fn it_converts_to_bytes() {
        let message: EventHubMessage = vec![1, 2, 3].into();
        let bytes: Vec<u8> = message.into();
        assert_eq!(bytes, [1, 2, 3]);
    }

    #[test]
    fn it_converts_to_body() {
        let message: EventHubMessage = "hello world!".into();
        let body: Body = message.into();
        assert_eq!(body.as_str().unwrap(), "hello world!");

        let message: EventHubMessage = json!({"hello": "world"}).into();
        let body: Body = message.into();
        assert_eq!(body.as_str().unwrap(), r#"{"hello":"world"}"#);

        let message: EventHubMessage = vec![1, 2, 3].into();
        let body: Body = message.into();
        assert_eq!(body.as_bytes(), [1, 2, 3]);
    }

    #[test]
    fn it_converts_to_typed_data() {
        let message: EventHubMessage = "test".into();
        let data: TypedData = message.into();
        assert_eq!(data.data, Some(Data::String("test".to_string())));

        let message: EventHubMessage = to_value("test").unwrap().into();
        let data: TypedData = message.into();
        assert_eq!(data.data, Some(Data::Json(r#""test""#.to_string())));

        let message: EventHubMessage = vec![1, 2, 3].into();
        let data: TypedData = message.into();
        assert_eq!(data.data, Some(Data::Bytes([1, 2, 3].to_vec())));
    }
}
