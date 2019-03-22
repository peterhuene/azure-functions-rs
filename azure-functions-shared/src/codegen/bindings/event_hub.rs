use azure_functions_shared_codegen::binding;
use std::borrow::Cow;

#[binding(name = "eventHub", direction = "out")]
pub struct EventHub {
    #[field(camel_case_value = true)]
    pub name: Cow<'static, str>,
    pub connection: Cow<'static, str>,
    #[field(name = "eventHubName")]
    pub event_hub_name: Option<Cow<'static, str>>,
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
        let binding = EventHub {
            name: Cow::from("foo"),
            connection: Cow::from("bar"),
            event_hub_name: Some(Cow::from("baz")),
        };

        assert_eq!(
            to_string(&binding).unwrap(),
            r#"{"type":"eventHub","direction":"out","name":"foo","connection":"bar","eventHubName":"baz"}"#
        );
    }

    #[test]
    fn it_parses_attribute_arguments() {
        let binding: EventHub = (
            vec![
                parse_str::<NestedMeta>(r#"name = "foo""#).unwrap(),
                parse_str::<NestedMeta>(r#"connection = "bar""#).unwrap(),
                parse_str::<NestedMeta>(r#"event_hub_name = "baz""#).unwrap(),
            ],
            Span::call_site(),
        )
            .into();

        assert_eq!(binding.name.as_ref(), "foo");
        assert_eq!(binding.connection.as_ref(), "bar");
        assert_eq!(binding.event_hub_name.as_ref().unwrap(), "baz");
    }

    #[test]
    fn it_requires_the_name_attribute_argument() {
        should_panic(
            || {
                let _: EventHub = (vec![], Span::call_site()).into();
            },
            "the 'name' argument is required for this binding",
        );
    }

    #[test]
    fn it_requires_the_name_attribute_be_a_string() {
        should_panic(
            || {
                let _: EventHub = (
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
                let _: EventHub = (
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
                let _: EventHub = (
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
                let _: EventHub = (
                    vec![parse_str::<NestedMeta>(r#"event_hub_name = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'event_hub_name' argument",
        );
    }

    #[test]
    fn it_converts_to_tokens() {
        let binding = EventHub {
            name: Cow::from("foo"),
            connection: Cow::from("bar"),
            event_hub_name: Some(Cow::from("baz")),
        };

        let mut stream = TokenStream::new();
        binding.to_tokens(&mut stream);
        let mut tokens = stream.to_string();
        tokens.retain(|c| c != ' ');

        assert_eq!(tokens, r#"::azure_functions::codegen::bindings::EventHub{name:::std::borrow::Cow::Borrowed("foo"),connection:::std::borrow::Cow::Borrowed("bar"),event_hub_name:Some(::std::borrow::Cow::Borrowed("baz")),}"#);
    }
}
