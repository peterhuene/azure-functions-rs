use azure_functions_shared_codegen::binding;
use std::borrow::Cow;

#[binding(name = "queue", direction = "out")]
pub struct Queue {
    #[field(camel_case_value = true)]
    pub name: Cow<'static, str>,
    #[field(name = "queueName")]
    pub queue_name: Cow<'static, str>,
    pub connection: Option<Cow<'static, str>>,
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
        let binding = Queue {
            name: Cow::from("foo"),
            queue_name: Cow::from("bar"),
            connection: Some(Cow::from("baz")),
        };

        assert_eq!(
            to_string(&binding).unwrap(),
            r#"{"type":"queue","direction":"out","name":"foo","queueName":"bar","connection":"baz"}"#
        );
    }

    #[test]
    fn it_parses_attribute_arguments() {
        let binding: Queue = (
            vec![
                parse_str::<NestedMeta>(r#"name = "foo""#).unwrap(),
                parse_str::<NestedMeta>(r#"queue_name = "bar""#).unwrap(),
                parse_str::<NestedMeta>(r#"connection = "baz""#).unwrap(),
            ],
            Span::call_site(),
        )
            .into();

        assert_eq!(binding.name.as_ref(), "foo");
        assert_eq!(binding.queue_name.as_ref(), "bar");
        assert_eq!(binding.connection.unwrap().as_ref(), "baz");
    }

    #[test]
    fn it_requires_the_name_attribute_argument() {
        should_panic(
            || {
                let _: Queue = (vec![], Span::call_site()).into();
            },
            "the 'name' argument is required for this binding",
        );
    }

    #[test]
    fn it_requires_the_name_attribute_be_a_string() {
        should_panic(
            || {
                let _: Queue = (
                    vec![parse_str::<NestedMeta>(r#"name = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'name' argument",
        );
    }

    #[test]
    fn it_requires_the_queue_name_attribute_argument() {
        should_panic(
            || {
                let _: Queue = (
                    vec![parse_str::<NestedMeta>(r#"name = "foo""#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "the 'queue_name' argument is required for this binding",
        );
    }

    #[test]
    fn it_requires_the_queue_name_attribute_be_a_string() {
        should_panic(
            || {
                let _: Queue = (
                    vec![parse_str::<NestedMeta>(r#"queue_name = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'queue_name' argument",
        );
    }

    #[test]
    fn it_requires_the_connection_attribute_be_a_string() {
        should_panic(
            || {
                let _: Queue = (
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
        let binding = Queue {
            name: Cow::from("foo"),
            queue_name: Cow::from("bar"),
            connection: Some(Cow::from("baz")),
        };

        let mut stream = TokenStream::new();
        binding.to_tokens(&mut stream);
        let mut tokens = stream.to_string();
        tokens.retain(|c| c != ' ');

        assert_eq!(tokens, r#"::azure_functions::codegen::bindings::Queue{name:::std::borrow::Cow::Borrowed("foo"),queue_name:::std::borrow::Cow::Borrowed("bar"),connection:Some(::std::borrow::Cow::Borrowed("baz")),}"#);
    }
}
