use rpc::protocol;
use serde_json;
use std::borrow::Cow;

/// Represents the body of a HTTP request or response.
#[derive(Debug)]
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
            Body::Json(_) => "application/json",
            Body::Bytes(_) => "application/octet-stream",
            _ => "text/plain",
        }
    }
}

#[doc(hidden)]
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

impl<'a> From<serde_json::Value> for Body<'a> {
    fn from(data: serde_json::Value) -> Self {
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
