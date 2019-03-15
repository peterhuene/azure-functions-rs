use azure_functions_shared_codegen::binding;
use std::borrow::Cow;

#[binding(name = "signalR", direction = "out")]
pub struct SignalR {
    #[field(camel_case_value = true)]
    pub name: Cow<'static, str>,
    #[field(name = "hubName")]
    pub hub_name: Cow<'static, str>,
    #[field(name = "connectionStringSetting")]
    pub connection: Option<Cow<'static, str>>,
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
        let binding = SignalR {
            name: Cow::from("foo"),
            hub_name: Cow::from("bar"),
            connection: Some(Cow::from("baz")),
        };

        assert_eq!(
            to_string(&binding).unwrap(),
            r#"{"type":"signalR","direction":"out","name":"foo","hubName":"bar","connectionStringSetting":"baz"}"#
        );
    }

    #[test]
    fn it_parses_attribute_arguments() {
        let binding: SignalR = (
            vec![
                parse_str::<NestedMeta>(r#"name = "foo""#).unwrap(),
                parse_str::<NestedMeta>(r#"hub_name = "bar""#).unwrap(),
                parse_str::<NestedMeta>(r#"connection = "baz""#).unwrap(),
            ],
            Span::call_site(),
        )
            .into();

        assert_eq!(binding.name.as_ref(), "foo");
        assert_eq!(binding.hub_name.as_ref(), "bar");
        assert_eq!(binding.connection.unwrap().as_ref(), "baz");
    }

    #[test]
    fn it_requires_the_name_attribute_argument() {
        should_panic(
            || {
                let _: SignalR = (vec![], Span::call_site()).into();
            },
            "the 'name' argument is required for this binding",
        );
    }

    #[test]
    fn it_requires_the_name_attribute_be_a_string() {
        should_panic(
            || {
                let _: SignalR = (
                    vec![parse_str::<NestedMeta>(r#"name = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'name' argument",
        );
    }

    #[test]
    fn it_requires_the_hub_name_attribute_argument() {
        should_panic(
            || {
                let _: SignalR = (
                    vec![parse_str::<NestedMeta>(r#"name = "foo""#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "the 'hub_name' argument is required for this binding",
        );
    }

    #[test]
    fn it_requires_the_queue_name_attribute_be_a_string() {
        should_panic(
            || {
                let _: SignalR = (
                    vec![parse_str::<NestedMeta>(r#"hub_name = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'hub_name' argument",
        );
    }

    #[test]
    fn it_requires_the_connection_attribute_be_a_string() {
        should_panic(
            || {
                let _: SignalR = (
                    vec![parse_str::<NestedMeta>(r#"connection = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'connection' argument",
        );
    }

    #[test]
    fn it_converts_to_tokens() {
        let binding = SignalR {
            name: Cow::from("foo"),
            hub_name: Cow::from("bar"),
            connection: Some(Cow::from("cake")),
        };

        let mut stream = TokenStream::new();
        binding.to_tokens(&mut stream);
        let mut tokens = stream.to_string();
        tokens.retain(|c| c != ' ');

        assert_eq!(
            tokens,
            r#"::azure_functions::codegen::bindings::SignalR{name:::std::borrow::Cow::Borrowed("foo"),hub_name:::std::borrow::Cow::Borrowed("bar"),connection:Some(::std::borrow::Cow::Borrowed("cake")),}"#
        );
    }
}
