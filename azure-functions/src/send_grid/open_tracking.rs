use serde::{Deserialize, Serialize};

/// Represents the ability to track whether a recipient opened an email message.
///
/// Open tracking includes a single pixel image in the body of the content to determine when the email was opened.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct OpenTracking {
    /// The value indicating whether this setting is enabled.
    pub enable: bool,
    /// The substitution tag that can be used to control the desired location of the tracking pixel in the email message.
    ///
    /// The tag will be replaced by the open tracking pixel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub substitution_tag: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;

    #[test]
    fn it_serializes_to_json() {
        let json = to_string(&OpenTracking {
            enable: true,
            substitution_tag: Some("foo".to_owned()),
        })
        .unwrap();

        assert_eq!(json, r#"{"enable":true,"substitution_tag":"foo"}"#);
    }
}
