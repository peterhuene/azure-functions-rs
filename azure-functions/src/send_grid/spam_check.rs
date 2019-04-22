use serde_derive::{Deserialize, Serialize};

/// Represents the ability to test the email message for spam content.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SpamCheck {
    /// The value indicating whether this setting is enabled.
    pub enable: bool,
    /// The threshold used to determine if your content qualifies as spam on a scale from 1 to 10.
    ///
    /// A value of 10 is the most strict or most likely to be considered as spam.
    pub threshold: i32,
    /// The inbound post URL that you would like a copy of your email, along with the spam report, sent to.
    ///
    /// The URL must start with `http://` or `https://`.
    pub post_to_url: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;

    #[test]
    fn it_serializes_to_json() {
        let json = to_string(&SpamCheck {
            enable: true,
            threshold: 7,
            post_to_url: "https://example.com".to_owned(),
        })
        .unwrap();

        assert_eq!(
            json,
            r#"{"enable":true,"threshold":7,"post_to_url":"https://example.com"}"#
        );
    }
}
