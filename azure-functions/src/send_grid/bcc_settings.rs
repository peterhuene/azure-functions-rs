use serde::{Deserialize, Serialize};

/// Represents bcc settings for an email message.
///
/// The specified email will receive a blind carbon copy (BCC) of
/// the very first personalization defined for an email message.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BccSettings {
    /// The value indicating whether this setting is enabled.
    pub enable: bool,
    /// The email address that will receive the BCC.
    pub email: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;

    #[test]
    fn it_serializes_to_json() {
        let json = to_string(&BccSettings {
            enable: true,
            email: "foo@example.com".to_owned(),
        })
        .unwrap();

        assert_eq!(json, r#"{"enable":true,"email":"foo@example.com"}"#);
    }
}
