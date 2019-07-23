use serde::{Deserialize, Serialize};

/// Represents footer settings for an email message.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FooterSettings {
    /// The value indicating whether this setting is enabled.
    pub enable: bool,
    /// The plain text content of the footer.
    pub text: String,
    /// The HTML content of the footer.
    pub html: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;

    #[test]
    fn it_serializes_to_json() {
        let json = to_string(&FooterSettings {
            enable: true,
            text: "hello".to_owned(),
            html: "world".to_owned(),
        })
        .unwrap();

        assert_eq!(json, r#"{"enable":true,"text":"hello","html":"world"}"#);
    }
}
