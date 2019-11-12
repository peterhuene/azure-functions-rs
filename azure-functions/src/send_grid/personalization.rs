use crate::send_grid::EmailAddress;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

/// Represents a personalization of an email message.
///
/// Defines who should receive an individual message and how that message should be handled.
///
/// Fields in personalizations will override the fields of the same name from the message level.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Personalization {
    /// The list of email recipients.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub to: Vec<EmailAddress>,
    /// The list of recipients who will receive a carbon copy of the email.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cc: Vec<EmailAddress>,
    /// The list of recipients who will receive a blind carbon copy of the email.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bcc: Vec<EmailAddress>,
    /// The subject line of the email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// The email message headers.
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
    /// The map of substitution tags to substitution values.
    /// Substitutions will apply to the content of the email, in addition to the subject and reply-to parameters.
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub substitutions: HashMap<String, String>,
    /// The map of custom argument to value.
    /// Custom arguments will be carried along with the email, activity data, and links.
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub custom_args: HashMap<String, String>,
    /// The unix timestamp specifying when the email should be sent from Twilio SendGrid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_at: Option<i64>,
    /// The template data to use for Handlebars based templates.
    #[serde(
        rename = "dynamic_template_data",
        skip_serializing_if = "Option::is_none"
    )]
    pub template_data: Option<Map<String, Value>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;

    #[test]
    fn it_serializes_to_json() {
        let mut headers = HashMap::new();
        headers.insert("foo".to_owned(), "bar".to_owned());

        let mut substitutions = HashMap::new();
        substitutions.insert("key".to_owned(), "value".to_owned());

        let mut custom_args = HashMap::new();
        custom_args.insert("baz".to_owned(), "jam".to_owned());

        let mut template_data = Map::<String, Value>::new();
        template_data.insert("hello".to_owned(), Value::String("world".to_owned()));

        let json = to_string(&Personalization {
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
            headers,
            substitutions,
            custom_args,
            send_at: Some(12345),
            template_data: Some(template_data),
        })
        .unwrap();

        assert_eq!(
            json,
            r#"{"to":[{"email":"foo@example.com"},{"email":"bar@example.com","name":"Bar Baz"}],"cc":[{"email":"baz@example.com"},{"email":"jam@example.com","name":"Jam"}],"bcc":[{"email":"cake@example.com"},{"email":"lie@example.com","name":"Lie"}],"subject":"hello world","headers":{"foo":"bar"},"substitutions":{"key":"value"},"custom_args":{"baz":"jam"},"send_at":12345,"dynamic_template_data":{"hello":"world"}}"#
        );
    }
}
