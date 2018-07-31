use rpc::protocol;
use serde::de::Error;
use serde::Deserialize;
use serde_json::{from_str, Result, Value};
use std::borrow::Cow;
use std::fmt;
use std::str::from_utf8;

/// Represents the contents of an Azure Storage blob.
#[derive(Clone, Debug)]
pub enum Contents<'a> {
    /// Represents a string blob.
    String(Cow<'a, str>),
    /// Represents a JSON blob.
    Json(Cow<'a, str>),
    /// Represents a blob of bytes.
    Bytes(Cow<'a, [u8]>),
}

impl Contents<'_> {
    /// Gets the contents of the blob as a string.
    ///
    /// Returns None if there is no valid string representation of the blob.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Contents::String(s) => Some(s),
            Contents::Json(s) => Some(s),
            Contents::Bytes(b) => from_utf8(b).map(|s| s).ok(),
        }
    }

    /// Gets the contents of the blob as a slice of bytes.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Contents::String(s) => s.as_bytes(),
            Contents::Json(s) => s.as_bytes(),
            Contents::Bytes(b) => b,
        }
    }

    /// Deserializes the blob as JSON to the requested type.
    pub fn from_json<T>(&'b self) -> Result<T>
    where
        T: Deserialize<'b>,
    {
        match self {
            Contents::String(s) => from_str(s.as_ref()),
            Contents::Json(s) => from_str(s.as_ref()),
            Contents::Bytes(b) => from_str(from_utf8(b).map_err(|e| {
                ::serde_json::Error::custom(format!("blob is not valid UTF-8: {}", e))
            })?),
        }
    }
}

impl fmt::Display for Contents<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str().unwrap_or(""))
    }
}

impl From<&'a protocol::TypedData> for Contents<'a> {
    fn from(data: &'a protocol::TypedData) -> Self {
        if data.has_string() {
            return Contents::String(Cow::Borrowed(data.get_string()));
        }
        if data.has_json() {
            return Contents::Json(Cow::Borrowed(data.get_json()));
        }
        if data.has_bytes() {
            return Contents::Bytes(Cow::Borrowed(data.get_bytes()));
        }
        if data.has_stream() {
            return Contents::Bytes(Cow::Borrowed(data.get_stream()));
        }

        panic!("unexpected data for blob contents");
    }
}

impl From<&'a str> for Contents<'a> {
    fn from(data: &'a str) -> Self {
        Contents::String(Cow::Borrowed(data))
    }
}

impl From<String> for Contents<'_> {
    fn from(data: String) -> Self {
        Contents::String(Cow::Owned(data))
    }
}

impl From<Value> for Contents<'_> {
    fn from(data: Value) -> Self {
        Contents::Json(Cow::Owned(data.to_string()))
    }
}

impl From<&'a [u8]> for Contents<'a> {
    fn from(data: &'a [u8]) -> Self {
        Contents::Bytes(Cow::Borrowed(data))
    }
}

impl From<Vec<u8>> for Contents<'_> {
    fn from(data: Vec<u8>) -> Self {
        Contents::Bytes(Cow::Owned(data))
    }
}

impl Into<protocol::TypedData> for Contents<'_> {
    fn into(self) -> protocol::TypedData {
        let mut data = protocol::TypedData::new();

        match self {
            Contents::String(s) => data.set_string(s.into_owned()),
            Contents::Json(s) => data.set_json(s.into_owned()),
            Contents::Bytes(b) => data.set_bytes(b.into_owned()),
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
    fn it_has_string_content() {
        const CONTENTS: &'static str = "test content";

        let contents: Contents = CONTENTS.into();
        assert_eq!(contents.as_str().unwrap(), CONTENTS);

        let data: protocol::TypedData = contents.into();
        assert_eq!(data.get_string(), CONTENTS);
    }

    #[test]
    fn it_has_json_contents() {
        #[derive(Serialize, Deserialize)]
        struct Data {
            message: String,
        };

        const MESSAGE: &'static str = "test";

        let data = Data {
            message: MESSAGE.to_string(),
        };

        let contents: Contents = ::serde_json::to_value(data).unwrap().into();
        assert_eq!(contents.from_json::<Data>().unwrap().message, MESSAGE);

        let data: protocol::TypedData = contents.into();
        assert_eq!(data.get_json(), r#"{"message":"test"}"#);
    }

    #[test]
    fn it_has_bytes_content() {
        const CONTENTS: &'static [u8] = &[1, 2, 3];

        let content: Contents = CONTENTS.into();
        assert_eq!(content.as_bytes(), CONTENTS);

        let data: protocol::TypedData = content.into();
        assert_eq!(data.get_bytes(), CONTENTS);
    }

    #[test]
    fn it_displays_as_a_string() {
        const CONTENTS: &'static str = "test";

        let content: Contents = CONTENTS.into();

        let mut s = String::new();
        write!(s, "{}", content);

        assert_eq!(s, CONTENTS);
    }

    #[test]
    fn it_converts_from_typed_data() {
        let mut data = protocol::TypedData::new();
        data.set_string("test".to_string());
        let content: Contents = (&data).into();
        assert!(matches!(content, Contents::String(_)));
        assert_eq!(content.as_str().unwrap(), "test");

        let mut data = protocol::TypedData::new();
        data.set_json("test".to_string());
        let content: Contents = (&data).into();
        assert!(matches!(content, Contents::Json(_)));
        assert_eq!(content.as_str().unwrap(), "test");

        let mut data = protocol::TypedData::new();
        data.set_bytes(vec![0, 1, 2]);
        let content: Contents = (&data).into();
        assert!(matches!(content, Contents::Bytes(_)));
        assert_eq!(content.as_bytes(), [0, 1, 2]);

        let mut data = protocol::TypedData::new();
        data.set_stream(vec![0, 1, 2]);
        let content: Contents = (&data).into();
        assert!(matches!(content, Contents::Bytes(_)));
        assert_eq!(content.as_bytes(), [0, 1, 2]);
    }

    #[test]
    fn it_converts_from_str() {
        let content: Contents = "test".into();
        assert!(matches!(content, Contents::String(Cow::Borrowed(_))));
        assert_eq!(content.as_str().unwrap(), "test");
    }

    #[test]
    fn it_converts_from_string() {
        let content: Contents = "test".to_string().into();
        assert!(matches!(content, Contents::String(Cow::Owned(_))));
        assert_eq!(content.as_str().unwrap(), "test");
    }

    #[test]
    fn it_converts_from_json() {
        let content: Contents = to_value("hello world").unwrap().into();
        assert!(matches!(content, Contents::Json(Cow::Owned(_))));
        assert_eq!(content.as_str().unwrap(), r#""hello world""#);
    }

    #[test]
    fn it_converts_from_u8_slice() {
        let content: Contents = [0, 1, 2][..].into();
        assert!(matches!(content, Contents::Bytes(Cow::Borrowed(_))));
        assert_eq!(content.as_bytes(), [0, 1, 2]);
    }

    #[test]
    fn it_converts_from_u8_vec() {
        let content: Contents = vec![0, 1, 2].into();
        assert!(matches!(content, Contents::Bytes(Cow::Owned(_))));
        assert_eq!(content.as_bytes(), [0, 1, 2]);
    }

    #[test]
    fn it_converts_to_typed_data() {
        let content: Contents = "test".into();
        let data: protocol::TypedData = content.into();
        assert!(data.has_string());
        assert_eq!(data.get_string(), "test");

        let content: Contents = to_value("test").unwrap().into();
        let data: protocol::TypedData = content.into();
        assert!(data.has_json());
        assert_eq!(data.get_json(), r#""test""#);

        let content: Contents = vec![1, 2, 3].into();
        let data: protocol::TypedData = content.into();
        assert!(data.has_bytes());
        assert_eq!(data.get_bytes(), [1, 2, 3]);
    }
}
