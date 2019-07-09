use crate::{
    http::Body,
    rpc::{typed_data::Data, TypedData},
};
use serde::de::Error;
use serde::Deserialize;
use serde_json::{from_str, Result, Value};
use std::borrow::Cow;
use std::fmt;
use std::str::from_utf8;

/// Represents an Azure Storage blob input or output binding.
///
/// The following binding attributes are supported:
///
/// | Name         | Description                                                                                                                        |
/// |--------------|------------------------------------------------------------------------------------------------------------------------------------|
/// | `name`       | The name of the parameter being bound.                                                                                             |
/// | `path`       | The path to the blob.                                                                                                              |
/// | `connection` | The name of an app setting that contains the Storage connection string to use for this binding. Defaults to `AzureWebJobsStorage`. |
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
pub struct Blob(TypedData);

impl Blob {
    /// Gets the content of the blob as a string.
    ///
    /// Returns None if there is no valid string representation of the blob.
    pub fn as_str(&self) -> Option<&str> {
        match &self.0.data {
            Some(Data::String(s)) => Some(s),
            Some(Data::Json(s)) => Some(s),
            Some(Data::Bytes(b)) => from_utf8(b).ok(),
            Some(Data::Stream(s)) => from_utf8(s).ok(),
            _ => None,
        }
    }

    /// Gets the content of the blob as a slice of bytes.
    pub fn as_bytes(&self) -> &[u8] {
        match &self.0.data {
            Some(Data::String(s)) => s.as_bytes(),
            Some(Data::Json(s)) => s.as_bytes(),
            Some(Data::Bytes(b)) => b,
            Some(Data::Stream(s)) => s,
            _ => panic!("unexpected data for blob content"),
        }
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
        Blob(TypedData {
            data: Some(Data::String(content.to_owned())),
        })
    }
}

impl From<String> for Blob {
    fn from(content: String) -> Self {
        Blob(TypedData {
            data: Some(Data::String(content)),
        })
    }
}

impl From<&Value> for Blob {
    fn from(content: &Value) -> Self {
        Blob(TypedData {
            data: Some(Data::Json(content.to_string())),
        })
    }
}

impl From<Value> for Blob {
    fn from(content: Value) -> Self {
        Blob(TypedData {
            data: Some(Data::Json(content.to_string())),
        })
    }
}

impl<'a> From<&'a [u8]> for Blob {
    fn from(content: &'a [u8]) -> Self {
        Blob(TypedData {
            data: Some(Data::Bytes(content.to_owned())),
        })
    }
}

impl From<Vec<u8>> for Blob {
    fn from(content: Vec<u8>) -> Self {
        Blob(TypedData {
            data: Some(Data::Bytes(content)),
        })
    }
}

#[doc(hidden)]
impl From<TypedData> for Blob {
    fn from(data: TypedData) -> Self {
        Blob(data)
    }
}

impl Into<String> for Blob {
    fn into(self) -> String {
        match self.0.data {
            Some(Data::String(s)) => s,
            Some(Data::Json(s)) => s,
            Some(Data::Bytes(b)) => {
                String::from_utf8(b).expect("blob does not contain valid UTF-8 bytes")
            }
            Some(Data::Stream(s)) => {
                String::from_utf8(s).expect("blob does not contain valid UTF-8 bytes")
            }
            _ => panic!("unexpected data for blob content"),
        }
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
    fn into(self) -> Vec<u8> {
        match self.0.data {
            Some(Data::String(s)) => s.into_bytes(),
            Some(Data::Json(s)) => s.into_bytes(),
            Some(Data::Bytes(b)) => b,
            Some(Data::Stream(s)) => s,
            _ => panic!("unexpected data for blob content"),
        }
    }
}

impl<'a> Into<Body<'a>> for Blob {
    fn into(self) -> Body<'a> {
        match self.0.data {
            Some(Data::String(s)) => s.into(),
            Some(Data::Json(s)) => Body::Json(Cow::from(s)),
            Some(Data::Bytes(b)) => b.into(),
            Some(Data::Stream(s)) => s.into(),
            _ => panic!("unexpected data for blob content"),
        }
    }
}

#[doc(hidden)]
impl Into<TypedData> for Blob {
    fn into(self) -> TypedData {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_derive::{Deserialize, Serialize};
    use serde_json::json;
    use serde_json::to_value;
    use std::fmt::Write;

    #[test]
    fn it_has_string_content() {
        const BLOB: &'static str = "test blob";

        let blob: Blob = BLOB.into();
        assert_eq!(blob.as_str().unwrap(), BLOB);

        let data: TypedData = blob.into();
        assert_eq!(data.data, Some(Data::String(BLOB.to_string())));
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

        let blob: Blob = ::serde_json::to_value(data).unwrap().into();
        assert_eq!(blob.as_json::<SerializedData>().unwrap().message, MESSAGE);

        let data: TypedData = blob.into();
        assert_eq!(
            data.data,
            Some(Data::Json(r#"{"message":"test"}"#.to_string()))
        );
    }

    #[test]
    fn it_has_bytes_content() {
        const BLOB: &'static [u8] = &[1, 2, 3];

        let blob: Blob = BLOB.into();
        assert_eq!(blob.as_bytes(), BLOB);

        let data: TypedData = blob.into();
        assert_eq!(data.data, Some(Data::Bytes(BLOB.to_owned())));
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

        let data = TypedData {
            data: Some(Data::String(BLOB.to_string())),
        };

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
        let data: TypedData = blob.into();
        assert_eq!(data.data, Some(Data::String("test".to_string())));

        let blob: Blob = to_value("test").unwrap().into();
        let data: TypedData = blob.into();
        assert_eq!(data.data, Some(Data::Json(r#""test""#.to_string())));

        let blob: Blob = vec![1, 2, 3].into();
        let data: TypedData = blob.into();
        assert_eq!(data.data, Some(Data::Bytes(vec![1, 2, 3])));
    }
}
