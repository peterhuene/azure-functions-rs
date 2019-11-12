use serde::{Deserialize, Serialize};

/// Represents the ability to enable tracking provided by Google Analytics.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GoogleAnalytics {
    /// The value indicating whether this setting is enabled.
    pub enable: bool,
    /// The name of the referrer source.
    #[serde(rename = "utm_source")]
    pub source: String,
    /// The name of the marketing medium.
    #[serde(rename = "utm_medium")]
    pub medium: String,
    /// The identification of any paid keywords.
    #[serde(rename = "utm_term")]
    pub term: String,
    /// The differentiation of your campaign from advertisements.
    #[serde(rename = "utm_content")]
    pub content: String,
    /// The name of the campaign.
    #[serde(rename = "utm_campaign")]
    pub campaign: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;

    #[test]
    fn it_serializes_to_json() {
        let json = to_string(&GoogleAnalytics {
            enable: true,
            source: "foo".to_owned(),
            medium: "bar".to_owned(),
            term: "baz".to_owned(),
            content: "jam".to_owned(),
            campaign: "cake".to_owned(),
        })
        .unwrap();

        assert_eq!(
            json,
            r#"{"enable":true,"utm_source":"foo","utm_medium":"bar","utm_term":"baz","utm_content":"jam","utm_campaign":"cake"}"#
        );
    }
}
