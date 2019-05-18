use crate::send_grid::{ClickTracking, GoogleAnalytics, OpenTracking, SubscriptionTracking};
use serde_derive::{Deserialize, Serialize};

/// Represents a collection of different mail settings that specify how an email message is handled.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TrackingSettings {
    /// The click tracking settings for the email message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub click_tracking: Option<ClickTracking>,
    /// The open tracking settings for the email message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_tracking: Option<OpenTracking>,
    /// The subscription tracking settings for the email message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_tracking: Option<SubscriptionTracking>,
    /// The Google Analytics settings for the email message.
    #[serde(rename = "ganalytics", skip_serializing_if = "Option::is_none")]
    pub google_analytics: Option<GoogleAnalytics>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;

    #[test]
    fn it_serializes_to_json() {
        let json = to_string(&TrackingSettings {
            click_tracking: Some(ClickTracking {
                enable: true,
                enable_text: false,
            }),
            open_tracking: Some(OpenTracking {
                enable: true,
                substitution_tag: Some("foo".to_owned()),
            }),
            subscription_tracking: Some(SubscriptionTracking {
                enable: true,
                text: "foo".to_owned(),
                html: "bar".to_owned(),
                substitution_tag: Some("baz".to_owned()),
            }),
            google_analytics: Some(GoogleAnalytics {
                enable: true,
                source: "foo".to_owned(),
                medium: "bar".to_owned(),
                term: "baz".to_owned(),
                content: "jam".to_owned(),
                campaign: "cake".to_owned(),
            }),
        })
        .unwrap();

        assert_eq!(json, r#"{"click_tracking":{"enable":true,"enable_text":false},"open_tracking":{"enable":true,"substitution_tag":"foo"},"subscription_tracking":{"enable":true,"text":"foo","html":"bar","substitution_tag":"baz"},"ganalytics":{"enable":true,"utm_source":"foo","utm_medium":"bar","utm_term":"baz","utm_content":"jam","utm_campaign":"cake"}}"#);
    }
}
