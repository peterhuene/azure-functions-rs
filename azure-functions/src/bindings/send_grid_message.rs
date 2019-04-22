use crate::{
    rpc::{typed_data::Data, TypedData},
    send_grid::{
        Attachment, Content, EmailAddress, MailSettings, Personalization, TrackingSettings,
        UnsubscribeGroup,
    },
    FromVec,
};
use serde_derive::{Deserialize, Serialize};
use serde_json::{to_string, to_value, Value};
use std::collections::HashMap;

/// Represents the SendGrid email message output binding.
///
/// The following binding attributes are supported:
///
/// | Name      | Description                                                                                                                                        |
/// |-----------|----------------------------------------------------------------------------------------------------------------------------------------------------|
/// | `api_key` | The name of an app setting that contains your API key. If not set, the default app setting name is "AzureWebJobsSendGridApiKey".
/// | `to`      | The default recipient's email address.
/// | `from`    | The default sender's email address.
/// | `subject` | The default subject of the email.
/// | `text`    | The default email text content.
///
/// # Examples
/// ```rust
/// use azure_functions::{
///     bindings::{HttpRequest, HttpResponse, SendGridMessage},
///     send_grid::MessageBuilder,
///     func,
/// };
///
/// #[func]
/// #[binding(name = "output1", from = "azure.functions.for.rust@example.com")]
/// pub fn send_email(req: HttpRequest) -> (HttpResponse, SendGridMessage) {
///     let params = req.query_params();
///
///     (
///         "The email was sent.".into(),
///         MessageBuilder::new()
///             .to(params.get("to").unwrap().as_str())
///             .subject(params.get("subject").unwrap().as_str())
///             .content(params.get("content").unwrap().as_str())
///             .build(),
///     )
/// }
/// ```
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SendGridMessage {
    /// The email address of the sender. If None, the `from` binding attribute is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<EmailAddress>,
    /// The subject of the email message. If None, the `subject` binding attribute is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// The list of personalized messages and their metadata.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub personalizations: Vec<Personalization>,
    /// The list of email content.
    #[serde(rename = "content", skip_serializing_if = "Vec::is_empty")]
    pub contents: Vec<Content>,
    /// The list of email attachments.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<Attachment>,
    /// The id of the SendGrid template to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_id: Option<String>,
    /// The map of key-value pairs of header names and the value to substitute for them.
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
    /// The map of key-value pairs that define large blocks of content that can be inserted into your emails using substitution tags.
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub sections: HashMap<String, String>,
    /// The list of category names for this message.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<String>,
    /// The map of key-value pairs that are specific to the entire send that will be carried along with the email and its activity data.
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub custom_args: HashMap<String, String>,
    /// The unix timestamp that specifies when the email should be sent from SendGrid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_at: Option<i64>,
    /// The associated unsubscribe group that specifies how to handle unsubscribes.
    #[serde(rename = "asm", skip_serializing_if = "Option::is_none")]
    pub unsubscribe_group: Option<UnsubscribeGroup>,
    /// The id that represents a batch of emails to be associated to each other for scheduling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<String>,
    /// The IP pool that the message should be sent from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_pool_name: Option<String>,
    /// The settings that specify how the email message should be handled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mail_settings: Option<MailSettings>,
    /// The settings that specify how the email message should be tracked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_settings: Option<TrackingSettings>,
    /// The email address and name of the individual who should receive responses to the email message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<EmailAddress>,
}

#[doc(hidden)]
impl Into<TypedData> for SendGridMessage {
    fn into(self) -> TypedData {
        TypedData {
            data: Some(Data::Json(
                to_string(&self).expect("failed to convert SendGrid message to JSON string"),
            )),
        }
    }
}

#[doc(hidden)]
impl FromVec<SendGridMessage> for TypedData {
    fn from_vec(vec: Vec<SendGridMessage>) -> Self {
        TypedData {
            data: Some(Data::Json(
                Value::Array(vec.into_iter().map(|m| to_value(m).unwrap()).collect()).to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::send_grid::{
        BccSettings, BypassListManagement, ClickTracking, FooterSettings, GoogleAnalytics,
        OpenTracking, SandboxMode, SpamCheck, SubscriptionTracking,
    };
    use serde_json::Map;

    #[test]
    fn it_serializes_to_json() {
        let mut headers = HashMap::new();
        headers.insert("foo".to_owned(), "bar".to_owned());

        let mut sections = HashMap::new();
        sections.insert("foo".to_owned(), "bar".to_owned());

        let mut substitutions = HashMap::new();
        substitutions.insert("key".to_owned(), "value".to_owned());

        let mut custom_args = HashMap::new();
        custom_args.insert("baz".to_owned(), "jam".to_owned());

        let mut template_data = Map::<String, Value>::new();
        template_data.insert("hello".to_owned(), Value::String("world".to_owned()));

        let json = to_string(&create_send_grid_message()).unwrap();

        assert_eq!(json, expected_message_json());
    }

    #[test]
    fn it_converts_to_typed_data() {
        let message = create_send_grid_message();

        let data: TypedData = message.into();
        assert_eq!(
            data.data,
            Some(Data::Json(expected_message_json().to_owned()))
        );
    }

    fn create_send_grid_message() -> SendGridMessage {
        let mut headers = HashMap::new();
        headers.insert("foo".to_owned(), "bar".to_owned());

        let mut sections = HashMap::new();
        sections.insert("foo".to_owned(), "bar".to_owned());

        let mut substitutions = HashMap::new();
        substitutions.insert("key".to_owned(), "value".to_owned());

        let mut custom_args = HashMap::new();
        custom_args.insert("baz".to_owned(), "jam".to_owned());

        let mut template_data = Map::<String, Value>::new();
        template_data.insert("hello".to_owned(), Value::String("world".to_owned()));

        SendGridMessage {
            from: Some(EmailAddress {
                email: "foo@example.com".to_owned(),
                name: Some("foo".to_owned()),
            }),
            subject: Some("hello world".to_owned()),
            personalizations: vec![Personalization {
                to: vec![
                    EmailAddress {
                        email: "foo@example.com".to_owned(),
                        ..Default::default()
                    },
                    EmailAddress {
                        email: "bar@example.com".to_owned(),
                        name: Some("Bar Baz".to_owned()),
                    },
                ],
                cc: vec![
                    EmailAddress {
                        email: "baz@example.com".to_owned(),
                        ..Default::default()
                    },
                    EmailAddress {
                        email: "jam@example.com".to_owned(),
                        name: Some("Jam".to_owned()),
                    },
                ],
                bcc: vec![
                    EmailAddress {
                        email: "cake@example.com".to_owned(),
                        ..Default::default()
                    },
                    EmailAddress {
                        email: "lie@example.com".to_owned(),
                        name: Some("Lie".to_owned()),
                    },
                ],
                subject: Some("hello world".to_owned()),
                headers: headers.clone(),
                substitutions,
                custom_args: custom_args.clone(),
                send_at: Some(12345),
                template_data: Some(template_data),
            }],
            contents: vec![Content {
                mime_type: "text/plain".to_owned(),
                value: "hello world".to_owned(),
            }],
            attachments: vec![Attachment {
                content: "aGVsbG8gd29ybGQ=".to_owned(),
                mime_type: "text/plain".to_owned(),
                filename: "foo.txt".to_owned(),
                disposition: None,
                content_id: None,
            }],
            template_id: Some("template".to_owned()),
            headers,
            sections,
            categories: vec!["first".to_owned(), "second".to_owned()],
            custom_args,
            send_at: Some(12345),
            unsubscribe_group: Some(UnsubscribeGroup {
                group_id: 12345,
                groups_to_display: Vec::new(),
            }),
            batch_id: Some("batch".to_owned()),
            ip_pool_name: Some("pool".to_owned()),
            mail_settings: Some(MailSettings {
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
            }),
            tracking_settings: Some(TrackingSettings {
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
            }),
            reply_to: Some(EmailAddress {
                email: "bar@example.com".to_owned(),
                name: Some("bar".to_owned()),
            }),
        }
    }

    fn expected_message_json() -> &'static str {
        r#"{"from":{"email":"foo@example.com","name":"foo"},"subject":"hello world","personalizations":[{"to":[{"email":"foo@example.com"},{"email":"bar@example.com","name":"Bar Baz"}],"cc":[{"email":"baz@example.com"},{"email":"jam@example.com","name":"Jam"}],"bcc":[{"email":"cake@example.com"},{"email":"lie@example.com","name":"Lie"}],"subject":"hello world","headers":{"foo":"bar"},"substitutions":{"key":"value"},"custom_args":{"baz":"jam"},"send_at":12345,"dynamic_template_data":{"hello":"world"}}],"content":[{"type":"text/plain","value":"hello world"}],"attachments":[{"content":"aGVsbG8gd29ybGQ=","type":"text/plain","filename":"foo.txt"}],"template_id":"template","headers":{"foo":"bar"},"sections":{"foo":"bar"},"categories":["first","second"],"custom_args":{"baz":"jam"},"send_at":12345,"asm":{"group_id":12345},"batch_id":"batch","ip_pool_name":"pool","mail_settings":{"bcc":{"enable":true,"email":"foo@example.com"},"bypass_list_management":{"enable":true},"footer":{"enable":true,"text":"hello","html":"world"},"sandbox_mode":{"enable":true},"spam_check":{"enable":true,"threshold":7,"post_to_url":"https://example.com"}},"tracking_settings":{"click_tracking":{"enable":true,"enable_text":false},"open_tracking":{"enable":true,"substitution_tag":"foo"},"subscription_tracking":{"enable":true,"text":"foo","html":"bar","substitution_tag":"baz"},"ganalytics":{"enable":true,"utm_source":"foo","utm_medium":"bar","utm_term":"baz","utm_content":"jam","utm_campaign":"cake"}},"reply_to":{"email":"bar@example.com","name":"bar"}}"#
    }
}
