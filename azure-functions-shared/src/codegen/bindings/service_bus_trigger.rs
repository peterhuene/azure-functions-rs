use azure_functions_shared_codegen::binding;
use std::borrow::Cow;

#[binding(name = "serviceBusTrigger", direction = "in", validate = "validate")]
pub struct ServiceBusTrigger {
    #[field(camel_case_value = true)]
    pub name: Cow<'static, str>,
    #[field(name = "queueName")]
    pub queue_name: Option<Cow<'static, str>>,
    #[field(name = "topicName")]
    pub topic_name: Option<Cow<'static, str>>,
    #[field(name = "subscriptionName")]
    pub subscription_name: Option<Cow<'static, str>>,
    pub connection: Option<Cow<'static, str>>,
}

impl ServiceBusTrigger {
    fn validate(&self) -> Result<(), String> {
        if self.queue_name.is_some() {
            if self.topic_name.is_some() || self.subscription_name.is_some() {
                return Err("service bus trigger binding cannot have both `queue_name` and either `topic_name` or `subscription_name` specified".to_owned());
            }
        } else if self.topic_name.is_none() || self.subscription_name.is_none() {
            return Err("service bus trigger binding must have either `queue_name` or both `topic_name` and `subscription_name` specified".to_owned());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::tests::should_panic;
    use proc_macro2::{Span, TokenStream};
    use quote::ToTokens;
    use serde_json::to_string;
    use syn::{parse_str, NestedMeta};

    #[test]
    fn it_serializes_to_json() {
        let binding = ServiceBusTrigger {
            name: Cow::from("foo"),
            queue_name: Some(Cow::from("bar")),
            topic_name: Some(Cow::from("baz")),
            subscription_name: Some(Cow::from("jam")),
            connection: Some(Cow::from("cake")),
        };

        assert_eq!(
            to_string(&binding).unwrap(),
            r#"{"type":"serviceBusTrigger","direction":"in","name":"foo","queueName":"bar","topicName":"baz","subscriptionName":"jam","connection":"cake"}"#
        );
    }

    #[test]
    fn it_parses_attribute_arguments() {
        let binding: ServiceBusTrigger = (
            vec![
                parse_str::<NestedMeta>(r#"name = "foo""#).unwrap(),
                parse_str::<NestedMeta>(r#"queue_name = "bar""#).unwrap(),
                parse_str::<NestedMeta>(r#"connection = "baz""#).unwrap(),
            ],
            Span::call_site(),
        )
            .into();

        assert_eq!(binding.name.as_ref(), "foo");
        assert_eq!(binding.queue_name.unwrap().as_ref(), "bar");
        assert!(binding.topic_name.is_none());
        assert!(binding.subscription_name.is_none());
        assert_eq!(binding.connection.unwrap().as_ref(), "baz");

        let binding: ServiceBusTrigger = (
            vec![
                parse_str::<NestedMeta>(r#"name = "foo""#).unwrap(),
                parse_str::<NestedMeta>(r#"topic_name = "bar""#).unwrap(),
                parse_str::<NestedMeta>(r#"subscription_name = "baz""#).unwrap(),
                parse_str::<NestedMeta>(r#"connection = "cake""#).unwrap(),
            ],
            Span::call_site(),
        )
            .into();

        assert_eq!(binding.name.as_ref(), "foo");
        assert!(binding.queue_name.is_none());
        assert_eq!(binding.topic_name.unwrap().as_ref(), "bar");
        assert_eq!(binding.subscription_name.unwrap().as_ref(), "baz");
        assert_eq!(binding.connection.unwrap().as_ref(), "cake");
    }

    #[test]
    fn it_requires_the_name_attribute_argument() {
        should_panic(
            || {
                let _: ServiceBusTrigger = (vec![], Span::call_site()).into();
            },
            "the 'name' argument is required for this binding",
        );
    }

    #[test]
    fn it_requires_the_name_attribute_be_a_string() {
        should_panic(
            || {
                let _: ServiceBusTrigger = (
                    vec![parse_str::<NestedMeta>(r#"name = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'name' argument",
        );
    }

    #[test]
    fn it_requires_the_queue_name_attribute_be_a_string() {
        should_panic(
            || {
                let _: ServiceBusTrigger = (
                    vec![parse_str::<NestedMeta>(r#"queue_name = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'queue_name' argument",
        );
    }

    #[test]
    fn it_requires_the_topic_name_attribute_be_a_string() {
        should_panic(
            || {
                let _: ServiceBusTrigger = (
                    vec![parse_str::<NestedMeta>(r#"topic_name = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'topic_name' argument",
        );
    }

    #[test]
    fn it_requires_the_subscription_name_attribute_be_a_string() {
        should_panic(
            || {
                let _: ServiceBusTrigger = (
                    vec![parse_str::<NestedMeta>(r#"subscription_name = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'subscription_name' argument",
        );
    }

    #[test]
    fn it_requires_the_connection_attribute_be_a_string() {
        should_panic(
            || {
                let _: ServiceBusTrigger = (
                    vec![parse_str::<NestedMeta>(r#"connection = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'connection' argument",
        );
    }

    #[test]
    fn it_requires_queue_name_or_topic_subscription() {
        should_panic(
            || {
                let _: ServiceBusTrigger = (
                    vec![
                        parse_str::<NestedMeta>(r#"name = "foo""#).unwrap(),
                    ],
                    Span::call_site(),
                )
                    .into();
            },
            "service bus trigger binding must have either `queue_name` or both `topic_name` and `subscription_name` specified",
        );

        should_panic(
            || {
                let _: ServiceBusTrigger = (
                    vec![
                        parse_str::<NestedMeta>(r#"name = "foo""#).unwrap(),
                        parse_str::<NestedMeta>(r#"queue_name = "bar""#).unwrap(),
                        parse_str::<NestedMeta>(r#"topic_name = "bar""#).unwrap(),
                    ],
                    Span::call_site(),
                )
                    .into();
            },
            "service bus trigger binding cannot have both `queue_name` and either `topic_name` or `subscription_name` specified",
        );

        should_panic(
            || {
                let _: ServiceBusTrigger = (
                    vec![
                        parse_str::<NestedMeta>(r#"name = "foo""#).unwrap(),
                        parse_str::<NestedMeta>(r#"queue_name = "bar""#).unwrap(),
                        parse_str::<NestedMeta>(r#"subscription_name = "bar""#).unwrap(),
                    ],
                    Span::call_site(),
                )
                    .into();
            },
            "service bus trigger binding cannot have both `queue_name` and either `topic_name` or `subscription_name` specified",
        );
    }

    #[test]
    fn it_converts_to_tokens() {
        let binding = ServiceBusTrigger {
            name: Cow::from("foo"),
            queue_name: Some(Cow::from("bar")),
            topic_name: Some(Cow::from("baz")),
            subscription_name: Some(Cow::from("jam")),
            connection: Some(Cow::from("cake")),
        };

        let mut stream = TokenStream::new();
        binding.to_tokens(&mut stream);
        let mut tokens = stream.to_string();
        tokens.retain(|c| c != ' ');

        assert_eq!(
            tokens,
            r#"::azure_functions::codegen::bindings::ServiceBusTrigger{name:::std::borrow::Cow::Borrowed("foo"),queue_name:Some(::std::borrow::Cow::Borrowed("bar")),topic_name:Some(::std::borrow::Cow::Borrowed("baz")),subscription_name:Some(::std::borrow::Cow::Borrowed("jam")),connection:Some(::std::borrow::Cow::Borrowed("cake")),}"#
        );
    }
}
