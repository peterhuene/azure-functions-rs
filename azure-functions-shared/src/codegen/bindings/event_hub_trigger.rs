use azure_functions_shared_codegen::binding;
use std::borrow::Cow;

#[binding(name = "eventHubTrigger", direction = "in")]
pub struct EventHubTrigger {
    #[field(camel_case_value = true)]
    pub name: Cow<'static, str>,
    pub connection: Cow<'static, str>,
    #[field(name = "eventHubName")]
    pub event_hub_name: Option<Cow<'static, str>>,
    #[field(name = "consumerGroup")]
    pub consumer_group: Option<Cow<'static, str>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::bindings::tests::should_panic;
    use proc_macro2::{Span, TokenStream};
    use quote::ToTokens;
    use serde_json::to_string;
    use syn::{parse_str, NestedMeta};

    #[test]
    fn it_serializes_to_json() {
        let binding = EventHubTrigger {
            name: Cow::from("foo"),
            connection: Cow::from("bar"),
            event_hub_name: Some(Cow::from("baz")),
            consumer_group: Some(Cow::from("cake")),
        };

        assert_eq!(
            to_string(&binding).unwrap(),
            r#"{"type":"eventHubTrigger","direction":"in","name":"foo","connection":"bar","eventHubName":"baz","consumerGroup":"cake"}"#
        );
    }

    #[test]
    fn it_parses_attribute_arguments() {
        let binding: EventHubTrigger = (
            vec![
                parse_str::<NestedMeta>(r#"name = "foo""#).unwrap(),
                parse_str::<NestedMeta>(r#"connection = "bar""#).unwrap(),
                parse_str::<NestedMeta>(r#"event_hub_name = "baz""#).unwrap(),
                parse_str::<NestedMeta>(r#"consumer_group = "cake""#).unwrap(),
            ],
            Span::call_site(),
        )
            .into();

        assert_eq!(binding.name.as_ref(), "foo");
        assert_eq!(binding.connection.as_ref(), "bar");
        assert_eq!(binding.event_hub_name.as_ref().unwrap(), "baz");
        assert_eq!(binding.consumer_group.as_ref().unwrap(), "cake");
    }

    #[test]
    fn it_requires_the_name_attribute_argument() {
        should_panic(
            || {
                let _: EventHubTrigger = (vec![], Span::call_site()).into();
            },
            "the 'name' argument is required for this binding",
        );
    }

    #[test]
    fn it_requires_the_name_attribute_be_a_string() {
        should_panic(
            || {
                let _: EventHubTrigger = (
                    vec![parse_str::<NestedMeta>(r#"name = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'name' argument",
        );
    }

    #[test]
    fn it_requires_the_connection_attribute_argument() {
        should_panic(
            || {
                let _: EventHubTrigger = (
                    vec![parse_str::<NestedMeta>(r#"name = "foo""#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "the 'connection' argument is required for this binding",
        );
    }

    #[test]
    fn it_requires_the_connection_attribute_be_a_string() {
        should_panic(
            || {
                let _: EventHubTrigger = (
                    vec![parse_str::<NestedMeta>(r#"connection = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'connection' argument",
        );
    }

    #[test]
    fn it_requires_the_event_hub_name_attribute_be_a_string() {
        should_panic(
            || {
                let _: EventHubTrigger = (
                    vec![parse_str::<NestedMeta>(r#"event_hub_name = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'event_hub_name' argument",
        );
    }

    #[test]
    fn it_requires_the_consumer_group_attribute_be_a_string() {
        should_panic(
            || {
                let _: EventHubTrigger = (
                    vec![parse_str::<NestedMeta>(r#"consumer_group = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'consumer_group' argument",
        );
    }

    #[test]
    fn it_converts_to_tokens() {
        let binding = EventHubTrigger {
            name: Cow::from("foo"),
            connection: Cow::from("bar"),
            event_hub_name: Some(Cow::from("baz")),
            consumer_group: Some(Cow::from("cake")),
        };

        let mut stream = TokenStream::new();
        binding.to_tokens(&mut stream);
        let mut tokens = stream.to_string();
        tokens.retain(|c| c != ' ');

        assert_eq!(tokens, r#"::azure_functions::codegen::bindings::EventHubTrigger{name:::std::borrow::Cow::Borrowed("foo"),connection:::std::borrow::Cow::Borrowed("bar"),event_hub_name:Some(::std::borrow::Cow::Borrowed("baz")),consumer_group:Some(::std::borrow::Cow::Borrowed("cake")),}"#);
    }
}
