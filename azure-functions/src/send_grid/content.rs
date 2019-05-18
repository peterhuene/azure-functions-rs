use serde_derive::{Deserialize, Serialize};

/// Represents the content of an email message.
///
/// You can include multiple mime types of content, but you must specify at least one.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Content {
    /// The mime type of the content you are including in your email (e.g. "text/plain" or "text/html").
    #[serde(rename = "type")]
    pub mime_type: String,
    /// The actual content of the specified mime type for the email message.
    pub value: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;

    #[test]
    fn it_serializes_to_json() {
        let json = to_string(&Content {
            mime_type: "text/plain".to_owned(),
            value: "hello world".to_owned(),
        })
        .unwrap();

        assert_eq!(json, r#"{"type":"text/plain","value":"hello world"}"#);
    }
}
