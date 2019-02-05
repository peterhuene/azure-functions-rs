use crate::rpc::protocol;
use serde::de::Error;
use serde::Deserialize;
use serde_json::{from_str, Result, Value};
use std::borrow::Cow;
use std::fmt;
use std::str::from_utf8;

/// Represents the body of a HTTP request or response.
#[derive(Clone, Debug)]
pub enum Body<'a> {
    /// Represents an empty body.
    Empty,
    /// Represents a string body with a default content type of `text/plain`.
    String(Cow<'a, str>),
    /// Represents a JSON body with a default content type of `application/json`.
    Json(Cow<'a, str>),
    /// Represents a body from a slice of bytes with a default content type of `application/octet-stream`.
    Bytes(Cow<'a, [u8]>),
}

impl Body<'_> {
    /// Gets the default content type for a body.
    ///
    /// Returns `application/json` for `Body::Json`.
    ///
    /// Returns `application/octet-stream` for `Body::Bytes`.
    ///
    /// Returns `text/plain` for all other `Body` values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use azure_functions::http::Body;
    ///
    /// let body: Body = [1, 2, 3][..].into();
    ///
    /// assert_eq!(body.default_content_type(), "application/octet-stream");
    /// ```
    pub fn default_content_type(&self) -> &str {
        match self {
            Body::Empty | Body::String(_) => "text/plain",
            Body::Json(_) => "application/json",
            Body::Bytes(_) => "application/octet-stream",
        }
    }

    /// Gets the body as a string.
    ///
    /// Returns None if there is no valid string representation of the message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use azure_functions::http::Body;
    /// use std::borrow::Cow;
    ///
    /// let body = Body::String(Cow::Borrowed("test"));
    /// assert_eq!(body.as_str().unwrap(), "test");
    /// ```
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Body::Empty => Some(""),
            Body::String(s) => Some(s),
            Body::Json(s) => Some(s),
            Body::Bytes(b) => from_utf8(b).map(|s| s).ok(),
        }
    }

    /// Gets the body as a slice of bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use azure_functions::http::Body;
    /// use std::borrow::Cow;
    ///
    /// let body = Body::String(Cow::Borrowed("test"));
    /// assert_eq!(body.as_bytes(), "test".as_bytes());
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Body::Empty => &[],
            Body::String(s) => s.as_bytes(),
            Body::Json(s) => s.as_bytes(),
            Body::Bytes(b) => b,
        }
    }

    /// Deserializes the body as JSON to the requested type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use] extern crate serde_derive;
    /// # extern crate serde;
    /// # extern crate azure_functions;
    /// use azure_functions::http::Body;
    /// use std::borrow::Cow;
    ///
    /// #[derive(Deserialize)]
    /// struct Data {
    ///     message: String
    /// }
    ///
    /// let body = Body::String(Cow::Borrowed(r#"{ "message": "hello" }"#));
    /// let data = body.as_json::<Data>().unwrap();
    /// assert_eq!(data.message, "hello");
    /// ```
    pub fn as_json<'b, T>(&'b self) -> Result<T>
    where
        T: Deserialize<'b>,
    {
        match self {
            Body::Empty => from_str(""),
            Body::String(s) => from_str(s.as_ref()),
            Body::Json(s) => from_str(s.as_ref()),
            Body::Bytes(b) => from_str(from_utf8(b).map_err(|e| {
                ::serde_json::Error::custom(format!("body is not valid UTF-8: {}", e))
            })?),
        }
    }
}

impl fmt::Display for Body<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str().unwrap_or(""))
    }
}

impl<'a> From<&'a protocol::TypedData> for Body<'a> {
    fn from(data: &'a protocol::TypedData) -> Self {
        if data.has_string() {
            return Body::String(Cow::Borrowed(data.get_string()));
        }
        if data.has_json() {
            return Body::Json(Cow::Borrowed(data.get_json()));
        }
        if data.has_bytes() {
            return Body::Bytes(Cow::Borrowed(data.get_bytes()));
        }
        if data.has_stream() {
            return Body::Bytes(Cow::Borrowed(data.get_stream()));
        }

        Body::Empty
    }
}

impl<'a> From<&'a str> for Body<'a> {
    fn from(data: &'a str) -> Self {
        Body::String(Cow::Borrowed(data))
    }
}

impl From<String> for Body<'_> {
    fn from(data: String) -> Self {
        Body::String(Cow::Owned(data))
    }
}

impl From<&Value> for Body<'_> {
    fn from(data: &Value) -> Self {
        Body::Json(Cow::Owned(data.to_string()))
    }
}

impl From<Value> for Body<'_> {
    fn from(data: Value) -> Self {
        Body::Json(Cow::Owned(data.to_string()))
    }
}

impl<'a> From<&'a [u8]> for Body<'a> {
    fn from(data: &'a [u8]) -> Self {
        Body::Bytes(Cow::Borrowed(data))
    }
}

impl From<Vec<u8>> for Body<'_> {
    fn from(data: Vec<u8>) -> Self {
        Body::Bytes(Cow::Owned(data))
    }
}

impl Into<protocol::TypedData> for Body<'_> {
    fn into(self) -> protocol::TypedData {
        let mut data = protocol::TypedData::new();

        match self {
            Body::Empty => {}
            Body::String(s) => data.set_string(s.into_owned()),
            Body::Json(s) => data.set_json(s.into_owned()),
            Body::Bytes(b) => data.set_bytes(b.into_owned()),
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
    fn it_has_a_default_content_type() {
        let body = Body::Empty;
        assert_eq!(body.default_content_type(), "text/plain");

        let body = Body::String(Cow::Borrowed("test"));
        assert_eq!(body.default_content_type(), "text/plain");

        let body = Body::Json(Cow::Borrowed("1"));
        assert_eq!(body.default_content_type(), "application/json");

        let body = Body::Bytes(Cow::Borrowed(&[]));
        assert_eq!(body.default_content_type(), "application/octet-stream");
    }

    #[test]
    fn it_has_a_string_body() {
        const BODY: &'static str = "test body";

        let body: Body = BODY.into();
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

        let body: Body = ::serde_json::to_value(data).unwrap().into();
        assert_eq!(body.as_json::<Data>().unwrap().message, MESSAGE);

        let data: protocol::TypedData = body.into();
        assert_eq!(data.get_json(), r#"{"message":"test"}"#);
    }

    #[test]
    fn it_has_a_bytes_body() {
        const BODY: &'static [u8] = &[1, 2, 3];

        let body: Body = BODY.into();
        assert_eq!(body.as_bytes(), BODY);

        let data: protocol::TypedData = body.into();
        assert_eq!(data.get_bytes(), BODY);
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
        let mut data = protocol::TypedData::new();
        data.set_string("test".to_string());
        let body: Body = (&data).into();
        assert!(matches!(body, Body::String(_)));
        assert_eq!(body.as_str().unwrap(), "test");

        let mut data = protocol::TypedData::new();
        data.set_json("test".to_string());
        let body: Body = (&data).into();
        assert!(matches!(body, Body::Json(_)));
        assert_eq!(body.as_str().unwrap(), "test");

        let mut data = protocol::TypedData::new();
        data.set_bytes(vec![0, 1, 2]);
        let body: Body = (&data).into();
        assert!(matches!(body, Body::Bytes(_)));
        assert_eq!(body.as_bytes(), [0, 1, 2]);

        let mut data = protocol::TypedData::new();
        data.set_stream(vec![0, 1, 2]);
        let body: Body = (&data).into();
        assert!(matches!(body, Body::Bytes(_)));
        assert_eq!(body.as_bytes(), [0, 1, 2]);
    }

    #[test]
    fn it_converts_from_str() {
        let body: Body = "test".into();
        assert!(matches!(body, Body::String(Cow::Borrowed(_))));
        assert_eq!(body.as_str().unwrap(), "test");
    }

    #[test]
    fn it_converts_from_string() {
        let body: Body = "test".to_string().into();
        assert!(matches!(body, Body::String(Cow::Owned(_))));
        assert_eq!(body.as_str().unwrap(), "test");
    }

    #[test]
    fn it_converts_from_json() {
        let body: Body = to_value("hello world").unwrap().into();
        assert!(matches!(body, Body::Json(Cow::Owned(_))));
        assert_eq!(body.as_str().unwrap(), r#""hello world""#);
    }

    #[test]
    fn it_converts_from_u8_slice() {
        let body: Body = [0, 1, 2][..].into();
        assert!(matches!(body, Body::Bytes(Cow::Borrowed(_))));
        assert_eq!(body.as_bytes(), [0, 1, 2]);
    }

    #[test]
    fn it_converts_from_u8_vec() {
        let body: Body = vec![0, 1, 2].into();
        assert!(matches!(body, Body::Bytes(Cow::Owned(_))));
        assert_eq!(body.as_bytes(), [0, 1, 2]);
    }

    #[test]
    fn it_converts_to_typed_data() {
        let body = Body::Empty;
        let data: protocol::TypedData = body.into();
        assert!(data.data.is_none());

        let body: Body = "test".into();
        let data: protocol::TypedData = body.into();
        assert!(data.has_string());
        assert_eq!(data.get_string(), "test");

        let body: Body = to_value("test").unwrap().into();
        let data: protocol::TypedData = body.into();
        assert!(data.has_json());
        assert_eq!(data.get_json(), r#""test""#);

        let body: Body = vec![1, 2, 3].into();
        let data: protocol::TypedData = body.into();
        assert!(data.has_bytes());
        assert_eq!(data.get_bytes(), [1, 2, 3]);
    }
}
