use serde::{Deserialize, Serialize};

/// Represents the ability to insert a subscription management link at the bottom of the text and html bodies of email messages.
///
/// If you would like to specify the location of the link within your email, use `substitution_tag`.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SubscriptionTracking {
    /// The value indicating whether this setting is enabled.
    pub enable: bool,
    /// The text to be appended to the email, with the subscription tracking link.
    pub text: String,
    /// The HTML to be appended to the email, with the subscription tracking link.
    pub html: String,
    /// The tag that will be replaced with the unsubscribe URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub substitution_tag: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;

    #[test]
    fn it_serializes_to_json() {
        let json = to_string(&SubscriptionTracking {
            enable: true,
            text: "foo".to_owned(),
            html: "bar".to_owned(),
            substitution_tag: Some("baz".to_owned()),
        })
        .unwrap();

        assert_eq!(
            json,
            r#"{"enable":true,"text":"foo","html":"bar","substitution_tag":"baz"}"#
        );
    }
}
