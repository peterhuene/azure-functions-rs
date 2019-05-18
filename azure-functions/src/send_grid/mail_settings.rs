use crate::send_grid::{BccSettings, BypassListManagement, FooterSettings, SandboxMode, SpamCheck};
use serde_derive::{Deserialize, Serialize};

/// Represents a collection of different mail settings that specify how an email message is handled.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MailSettings {
    /// The BCC settings for the email message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcc: Option<BccSettings>,
    /// The bypass list management settings for the email message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bypass_list_management: Option<BypassListManagement>,
    /// The footer settings for the email message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footer: Option<FooterSettings>,
    /// The sandbox mode settings for the email message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_mode: Option<SandboxMode>,
    /// The spam check settings for the email message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spam_check: Option<SpamCheck>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;

    #[test]
    fn it_serializes_to_json() {
        let json = to_string(&MailSettings {
            bcc: Some(BccSettings {
                enable: true,
                email: "foo@example.com".to_owned(),
            }),
            bypass_list_management: Some(BypassListManagement { enable: true }),
            footer: Some(FooterSettings {
                enable: true,
                text: "hello".to_owned(),
                html: "world".to_owned(),
            }),
            sandbox_mode: Some(SandboxMode { enable: true }),
            spam_check: Some(SpamCheck {
                enable: true,
                threshold: 7,
                post_to_url: "https://example.com".to_owned(),
            }),
        })
        .unwrap();

        assert_eq!(json, r#"{"bcc":{"enable":true,"email":"foo@example.com"},"bypass_list_management":{"enable":true},"footer":{"enable":true,"text":"hello","html":"world"},"sandbox_mode":{"enable":true},"spam_check":{"enable":true,"threshold":7,"post_to_url":"https://example.com"}}"#);
    }
}
