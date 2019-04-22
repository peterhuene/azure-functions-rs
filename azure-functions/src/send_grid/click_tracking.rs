use serde_derive::{Deserialize, Serialize};

/// Represents the ability to track whether a recipient clicked a link in the email message.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ClickTracking {
    /// The value indicating whether this setting is enabled.
    pub enable: bool,
    /// The value indicating if this setting should be included in the text/plain portion of the email message.
    pub enable_text: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;

    #[test]
    fn it_serializes_to_json() {
        let json = to_string(&ClickTracking {
            enable: true,
            enable_text: false,
        })
        .unwrap();

        assert_eq!(json, r#"{"enable":true,"enable_text":false}"#);
    }
}
