use crate::rpc::{typed_data::Data, TypedData};
use serde_json::{from_slice, Value};
use std::fmt;
use std::str::from_utf8;

/// Represents the body of a HTTP request or response.
#[derive(Clone, Debug)]
pub struct Body(TypedData);

impl Body {
    /// Gets the default content type for a body.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use azure_functions::http::Body;
    /// let body: Body = [1, 2, 3][..].into();
    /// assert_eq!(body.default_content_type(), "application/octet-stream");
    /// ```
    pub fn default_content_type(&self) -> &str {
        match &self.0.data {
            Some(Data::Json(_)) => "application/json",
            Some(Data::Bytes(_)) => "application/octet-stream",
            Some(Data::Stream(_)) => "application/octet-stream",
            _ => "text/plain",
        }
    }

    /// Gets the body as a string.
    ///
    /// Returns None if there is no valid string representation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use azure_functions::http::Body;
    /// let body: Body = "test".into();
    /// assert_eq!(body.to_str().unwrap(), "test");
    /// ```
    pub fn to_str(&self) -> Option<&str> {
        match &self.0.data {
            None => Some(""),
            Some(Data::String(s)) => Some(s),
            Some(Data::Json(s)) => Some(s),
            Some(Data::Bytes(b)) => from_utf8(b).ok(),
            Some(Data::Stream(s)) => from_utf8(s).ok(),
            _ => None,
        }
    }

    /// Gets the body as a slice of bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use azure_functions::http::Body;
    /// let body: Body = "example".into();
    /// assert_eq!(body.as_bytes(), "example".as_bytes());
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        match &self.0.data {
            None => &[],
            Some(Data::String(s)) => s.as_bytes(),
            Some(Data::Json(s)) => s.as_bytes(),
            Some(Data::Bytes(b)) => b,
            Some(Data::Stream(s)) => s,
            _ => panic!("unexpected data type for body"),
        }
    }
}

impl Default for Body {
    fn default() -> Self {
        Self(TypedData { data: None })
    }
}

impl fmt::Display for Body {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str().unwrap_or("<invalid-ut8>"))
    }
}

#[doc(hidden)]
impl From<TypedData> for Body {
    fn from(data: TypedData) -> Self {
        Self(data)
    }
}

impl From<&str> for Body {
    fn from(s: &str) -> Self {
        Self(TypedData {
            data: Some(Data::String(s.to_owned())),
        })
    }
}

impl From<String> for Body {
    fn from(s: String) -> Self {
        Self(TypedData {
            data: Some(Data::String(s)),
        })
    }
}

impl From<&Value> for Body {
    fn from(v: &Value) -> Self {
        Self(TypedData {
            data: Some(Data::Json(v.to_string())),
        })
    }
}

impl From<Value> for Body {
    fn from(v: Value) -> Self {
        Self(TypedData {
            data: Some(Data::Json(v.to_string())),
        })
    }
}

impl From<&[u8]> for Body {
    fn from(d: &[u8]) -> Self {
        Self(TypedData {
            data: Some(Data::Bytes(d.to_owned())),
        })
    }
}

impl From<Vec<u8>> for Body {
    fn from(d: Vec<u8>) -> Self {
        Self(TypedData {
            data: Some(Data::Bytes(d)),
        })
    }
}

impl Into<String> for Body {
    fn into(self) -> String {
        match self.0.data {
            Some(Data::String(s)) => s,
            Some(Data::Json(s)) => s,
            Some(Data::Bytes(b)) => {
                String::from_utf8(b).expect("body does not contain valid UTF-8 bytes")
            }
            Some(Data::Stream(s)) => {
                String::from_utf8(s).expect("body does not contain valid UTF-8 bytes")
            }
            _ => panic!("unexpected data for body content"),
        }
    }
}

impl Into<Value> for Body {
    fn into(self) -> Value {
        from_slice(self.as_bytes()).expect("body does not contain valid JSON data")
    }
}

impl Into<Vec<u8>> for Body {
    fn into(self) -> Vec<u8> {
        match self.0.data {
            Some(Data::String(s)) => s.into_bytes(),
            Some(Data::Json(s)) => s.into_bytes(),
            Some(Data::Bytes(b)) => b,
            Some(Data::Stream(s)) => s,
            _ => panic!("unexpected data for body content"),
        }
    }
}

#[doc(hidden)]
impl Into<TypedData> for Body {
    fn into(self) -> TypedData {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use serde_json::to_value;
    use std::fmt::Write;

    #[test]
    fn it_has_a_default_content_type() {
        let body: Body = "test".into();
        assert_eq!(body.default_content_type(), "text/plain");

        let body: Body = to_value(1).unwrap().into();
        assert_eq!(body.default_content_type(), "application/json");

        let body: Body = (&[] as &[u8]).into();
        assert_eq!(body.default_content_type(), "application/octet-stream");
    }

    #[test]
    fn it_has_a_string_body() {
        const BODY: &'static str = "test body";

        let body: Body = BODY.into();
        assert_eq!(body.to_str().unwrap(), BODY);

        let data: TypedData = body.into();
        assert_eq!(data.data, Some(Data::String(BODY.to_string())));
    }

    #[test]
    fn it_has_a_json_body() {
        #[derive(Serialize, Deserialize)]
        struct SerializedData {
            message: String,
        };

        const MESSAGE: &'static str = "test";

        let data = SerializedData {
            message: MESSAGE.to_string(),
        };

        let body: Body = to_value(data).unwrap().into();
        let data: SerializedData = from_slice(body.as_bytes()).unwrap();
        assert_eq!(data.message, MESSAGE);

        let data: TypedData = body.into();
        assert_eq!(
            data.data,
            Some(Data::Json(r#"{"message":"test"}"#.to_string()))
        );
    }

    #[test]
    fn it_has_a_bytes_body() {
        const BODY: &'static [u8] = &[1, 2, 3];

        let body: Body = BODY.into();
        assert_eq!(body.as_bytes(), BODY);

        let data: TypedData = body.into();
        assert_eq!(data.data, Some(Data::Bytes(BODY.to_vec())));
    }

    #[test]
    fn it_displays_as_a_string() {
        const BODY: &'static str = "test";

        let body: Body = BODY.into();

        let mut s = String::new();
        write!(s, "{}", body).unwrap();

        assert_eq!(s, BODY);
    }

    #[test]
    fn it_converts_from_typed_data() {
        let data = TypedData {
            data: Some(Data::String("test".to_string())),
        };

        let body: Body = data.into();
        assert_eq!(body.to_str().unwrap(), "test");

        let data = TypedData {
            data: Some(Data::Json("test".to_string())),
        };
        let body: Body = data.into();
        assert_eq!(body.to_str().unwrap(), "test");

        let data = TypedData {
            data: Some(Data::Bytes([0, 1, 2].to_vec())),
        };
        let body: Body = data.into();
        assert_eq!(body.as_bytes(), [0, 1, 2]);

        let data = TypedData {
            data: Some(Data::Stream([0, 1, 2].to_vec())),
        };
        let body: Body = data.into();
        assert_eq!(body.as_bytes(), [0, 1, 2]);
    }

    #[test]
    fn it_converts_from_str() {
        let body: Body = "test".into();
        assert_eq!(body.to_str().unwrap(), "test");
    }

    #[test]
    fn it_converts_from_string() {
        let body: Body = "test".to_string().into();
        assert_eq!(body.to_str().unwrap(), "test");
    }

    #[test]
    fn it_converts_from_json() {
        let body: Body = to_value("hello world").unwrap().into();
        assert_eq!(body.to_str().unwrap(), "\"hello world\"");
    }

    #[test]
    fn it_converts_from_u8_slice() {
        let body: Body = [0, 1, 2][..].into();
        assert_eq!(body.as_bytes(), [0, 1, 2]);
    }

    #[test]
    fn it_converts_from_u8_vec() {
        let body: Body = vec![0, 1, 2].into();
        assert_eq!(body.as_bytes(), [0, 1, 2]);
    }

    #[test]
    fn it_converts_to_typed_data() {
        let body: Body = "test".into();
        let data: TypedData = body.into();
        assert_eq!(data.data, Some(Data::String("test".to_string())));

        let body: Body = to_value("test").unwrap().into();
        let data: TypedData = body.into();
        assert_eq!(data.data, Some(Data::Json(r#""test""#.to_string())));

        let body: Body = vec![1, 2, 3].into();
        let data: TypedData = body.into();
        assert_eq!(data.data, Some(Data::Bytes([1, 2, 3].to_vec())));
    }
}
