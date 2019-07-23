use serde::{Deserialize, Serialize};

/// Represents an email attachment.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// The Base64 encoded content of the attachment.
    pub content: String,
    /// The mime type of the attachment (e.g. "image/jpeg").
    #[serde(rename = "type")]
    pub mime_type: String,
    /// The filename of the attachment.
    pub filename: String,
    /// The content-disposition of the attachment specifying how you would like the attachment to be displayed.
    ///
    /// Supported values are "attachment" or "inline".  Defaults to "attachment".
    ///
    /// For example, "inline" results in the attached file being displayed automatically within the message,
    /// while "attachment" results in the attached file requiring some action to be taken before it is displayed (e.g. opening or downloading the file).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disposition: Option<String>,
    /// The attachment's unique content identifier.
    ///
    /// This is used when the disposition is set to "inline" and the attachment is an image, allowing the file to be displayed within the body of your email.
    ///
    /// ```html
    /// <img src="cid:ii_139db99fdb5c3704"></img>
    /// ```
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;

    #[test]
    fn it_serializes_to_json() {
        let json = to_string(&Attachment {
            content: "aGVsbG8gd29ybGQ=".to_owned(),
            mime_type: "text/plain".to_owned(),
            filename: "foo.txt".to_owned(),
            disposition: None,
            content_id: None,
        })
        .unwrap();

        assert_eq!(
            json,
            r#"{"content":"aGVsbG8gd29ybGQ=","type":"text/plain","filename":"foo.txt"}"#
        );

        let json = to_string(&Attachment {
            content: "aGVsbG8gd29ybGQ=".to_owned(),
            mime_type: "text/plain".to_owned(),
            filename: "foo.txt".to_owned(),
            disposition: Some("inline".to_owned()),
            content_id: Some("123456".to_owned()),
        })
        .unwrap();

        assert_eq!(json, r#"{"content":"aGVsbG8gd29ybGQ=","type":"text/plain","filename":"foo.txt","disposition":"inline","content_id":"123456"}"#);
    }
}
