use crate::http::Body;
use crate::rpc::protocol;
use serde::de::Error;
use serde::Deserialize;
use serde_json::{from_str, Result, Value};
use std::borrow::Cow;
use std::fmt;
use std::str::from_utf8;

/// Represents an Azure Storage blob input or output binding.
///
/// # Examples
///
/// Creating a blob from a string:
///
/// ```rust
/// use azure_functions::bindings::{HttpRequest, Blob};
/// use azure_functions::func;
///
/// #[func]
/// #[binding(name = "output1", path = "example")]
/// pub fn create_blob(_req: HttpRequest) -> ((), Blob) {
///     ((), "Hello world!".into())
/// }
/// ```
///
/// Creating a blob from a JSON value (see the [json! macro](https://docs.serde.rs/serde_json/macro.json.html) from the `serde_json` crate):
///
/// ```rust
/// use azure_functions::bindings::{HttpRequest, Blob};
/// use azure_functions::func;
/// use serde_json::json;
///
/// #[func]
/// #[binding(name = "output1", path = "example")]
/// pub fn create_blob(_req: HttpRequest) -> ((), Blob) {
///     ((), json!({ "hello": "world!" }).into())
/// }
/// ```
///
/// Creating a blob from a sequence of bytes:
///
/// ```rust
/// use azure_functions::bindings::{HttpRequest, Blob};
/// use azure_functions::func;
///
/// #[func]
/// #[binding(name = "output1", path = "example")]
/// pub fn create_blob(_req: HttpRequest) -> ((), Blob) {
///     ((), [1, 2, 3][..].into())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Blob(protocol::TypedData);

impl Blob {
    /// Gets the content of the blob as a string.
    ///
    /// Returns None if there is no valid string representation of the blob.
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

    /// Gets the content of the blob as a slice of bytes.
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

        panic!("unexpected data for blob content");
    }

    /// Deserializes the blob as JSON to the requested type.
    pub fn as_json<'b, T>(&'b self) -> Result<T>
    where
        T: Deserialize<'b>,
    {
        from_str(
            self.as_str()
                .ok_or_else(|| ::serde_json::Error::custom("blob is not valid UTF-8"))?,
        )
    }
}

impl fmt::Display for Blob {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str().unwrap_or(""))
    }
}

impl<'a> From<&'a str> for Blob {
    fn from(content: &'a str) -> Self {
        let mut data = protocol::TypedData::new();
        data.set_string(content.to_owned());
        Blob(data)
    }
}

impl From<String> for Blob {
    fn from(content: String) -> Self {
        let mut data = protocol::TypedData::new();
        data.set_string(content);
        Blob(data)
    }
}

impl From<&Value> for Blob {
    fn from(content: &Value) -> Self {
        let mut data = protocol::TypedData::new();
        data.set_json(content.to_string());
        Blob(data)
    }
}

impl From<Value> for Blob {
    fn from(content: Value) -> Self {
        let mut data = protocol::TypedData::new();
        data.set_json(content.to_string());
        Blob(data)
    }
}

impl<'a> From<&'a [u8]> for Blob {
    fn from(content: &'a [u8]) -> Self {
        let mut data = protocol::TypedData::new();
        data.set_bytes(content.to_owned());
        Blob(data)
    }
}

impl From<Vec<u8>> for Blob {
    fn from(content: Vec<u8>) -> Self {
        let mut data = protocol::TypedData::new();
        data.set_bytes(content);
        Blob(data)
    }
}

#[doc(hidden)]
impl From<protocol::TypedData> for Blob {
    fn from(data: protocol::TypedData) -> Self {
        Blob(data)
    }
}

impl Into<String> for Blob {
    fn into(mut self) -> String {
        if self.0.has_string() {
            return self.0.take_string();
        }
        if self.0.has_json() {
            return self.0.take_json();
        }
        if self.0.has_bytes() {
            return String::from_utf8(self.0.take_bytes())
                .expect("blob does not contain valid UTF-8 bytes");
        }
        if self.0.has_stream() {
            return String::from_utf8(self.0.take_stream())
                .expect("blob does not contain valid UTF-8 bytes");
        }
        panic!("unexpected data for blob content");
    }
}

impl Into<Value> for Blob {
    fn into(self) -> Value {
        from_str(
            self.as_str()
                .expect("blob does not contain valid UTF-8 data"),
        )
        .expect("blob does not contain valid JSON data")
    }
}

impl Into<Vec<u8>> for Blob {
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

        panic!("unexpected data for blob content");
    }
}

impl<'a> Into<Body<'a>> for Blob {
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
impl Into<protocol::TypedData> for Blob {
    fn into(self) -> protocol::TypedData {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use serde_json::json;
    use serde_json::to_value;
    use std::fmt::Write;

    #[test]
    fn it_has_string_content() {
        const BLOB: &'static str = "test blob";

        let blob: Blob = BLOB.into();
        assert_eq!(blob.as_str().unwrap(), BLOB);

        let data: protocol::TypedData = blob.into();
        assert_eq!(data.get_string(), BLOB);
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

        let blob: Blob = ::serde_json::to_value(data).unwrap().into();
        assert_eq!(blob.as_json::<Data>().unwrap().message, MESSAGE);

        let data: protocol::TypedData = blob.into();
        assert_eq!(data.get_json(), r#"{"message":"test"}"#);
    }

    #[test]
    fn it_has_bytes_content() {
        const BLOB: &'static [u8] = &[1, 2, 3];

        let blob: Blob = BLOB.into();
        assert_eq!(blob.as_bytes(), BLOB);

        let data: protocol::TypedData = blob.into();
        assert_eq!(data.get_bytes(), BLOB);
    }

    #[test]
    fn it_displays_as_a_string() {
        const BLOB: &'static str = "test";

        let blob: Blob = BLOB.into();

        let mut s = String::new();
        write!(s, "{}", blob).unwrap();

        assert_eq!(s, BLOB);
    }

    #[test]
    fn it_converts_from_str() {
        let blob: Blob = "test".into();
        assert_eq!(blob.as_str().unwrap(), "test");
    }

    #[test]
    fn it_converts_from_string() {
        let blob: Blob = "test".to_string().into();
        assert_eq!(blob.as_str().unwrap(), "test");
    }

    #[test]
    fn it_converts_from_json() {
        let blob: Blob = to_value("hello world").unwrap().into();
        assert_eq!(blob.as_str().unwrap(), r#""hello world""#);
    }

    #[test]
    fn it_converts_from_u8_slice() {
        let blob: Blob = [0, 1, 2][..].into();
        assert_eq!(blob.as_bytes(), [0, 1, 2]);
    }

    #[test]
    fn it_converts_from_u8_vec() {
        let blob: Blob = vec![0, 1, 2].into();
        assert_eq!(blob.as_bytes(), [0, 1, 2]);
    }

    #[test]
    fn it_converts_from_typed_data() {
        const BLOB: &'static str = "hello world!";

        let mut data = protocol::TypedData::new();
        data.set_string(BLOB.to_string());

        let blob: Blob = data.into();
        assert_eq!(blob.as_str().unwrap(), BLOB);
    }

    #[test]
    fn it_converts_to_string() {
        let blob: Blob = "hello world!".into();
        let s: String = blob.into();
        assert_eq!(s, "hello world!");
    }

    #[test]
    fn it_converts_to_json() {
        let blob: Blob = json!({"hello": "world"}).into();
        let value: Value = blob.into();
        assert_eq!(value.to_string(), r#"{"hello":"world"}"#);
    }

    #[test]
    fn it_converts_to_bytes() {
        let blob: Blob = vec![1, 2, 3].into();
        let bytes: Vec<u8> = blob.into();
        assert_eq!(bytes, [1, 2, 3]);
    }

    #[test]
    fn it_converts_to_body() {
        let blob: Blob = "hello world!".into();
        let body: Body = blob.into();
        assert_eq!(body.as_str().unwrap(), "hello world!");

        let blob: Blob = json!({"hello": "world"}).into();
        let body: Body = blob.into();
        assert_eq!(body.as_str().unwrap(), r#"{"hello":"world"}"#);

        let blob: Blob = vec![1, 2, 3].into();
        let body: Body = blob.into();
        assert_eq!(body.as_bytes(), [1, 2, 3]);
    }

    #[test]
    fn it_converts_to_typed_data() {
        let blob: Blob = "test".into();
        let data: protocol::TypedData = blob.into();
        assert!(data.has_string());
        assert_eq!(data.get_string(), "test");

        let blob: Blob = to_value("test").unwrap().into();
        let data: protocol::TypedData = blob.into();
        assert!(data.has_json());
        assert_eq!(data.get_json(), r#""test""#);

        let blob: Blob = vec![1, 2, 3].into();
        let data: protocol::TypedData = blob.into();
        assert!(data.has_bytes());
        assert_eq!(data.get_bytes(), [1, 2, 3]);
    }
}
