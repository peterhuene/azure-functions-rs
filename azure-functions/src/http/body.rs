use rpc::protocol;
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

impl<'a> Body<'a> {
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
    /// let body: Body = [1u8, 2u8, 3u8][..].into();
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
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Body::Empty => Some(""),
            Body::String(s) => Some(s),
            Body::Json(s) => Some(s),
            Body::Bytes(b) => from_utf8(b).map(|s| s).ok(),
        }
    }

    /// Gets the body as a slice of bytes.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Body::Empty => &[],
            Body::String(s) => s.as_bytes(),
            Body::Json(s) => s.as_bytes(),
            Body::Bytes(b) => b,
        }
    }

    /// Deserializes the body as JSON to the requested type.
    pub fn from_json<'b, T>(&'b self) -> Result<T>
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

impl<'a> fmt::Display for Body<'a> {
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

impl<'a> From<String> for Body<'a> {
    fn from(data: String) -> Self {
        Body::String(Cow::Owned(data))
    }
}

impl<'a> From<Value> for Body<'a> {
    fn from(data: Value) -> Self {
        Body::Json(Cow::Owned(data.to_string()))
    }
}

impl<'a> From<&'a [u8]> for Body<'a> {
    fn from(data: &'a [u8]) -> Self {
        Body::Bytes(Cow::Borrowed(data))
    }
}

impl<'a> From<Vec<u8>> for Body<'a> {
    fn from(data: Vec<u8>) -> Self {
        Body::Bytes(Cow::Owned(data))
    }
}

impl<'a> Into<protocol::TypedData> for Body<'a> {
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
