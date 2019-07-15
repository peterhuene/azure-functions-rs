use crate::codegen::bindings::Direction;
use azure_functions_shared_codegen::binding;
use std::borrow::Cow;

#[binding(name = "blob")]
pub struct Blob {
    pub direction: Direction,
    #[field(camel_case_value = true)]
    pub name: Cow<'static, str>,
    pub path: Cow<'static, str>,
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
        let binding = Blob {
            direction: Direction::Out,
            name: Cow::from("foo"),
            path: Cow::from("bar"),
            connection: Some(Cow::from("baz")),
        };

        assert_eq!(
            to_string(&binding).unwrap(),
            r#"{"type":"blob","direction":"out","name":"foo","path":"bar","connection":"baz"}"#
        );

        let binding = Blob {
            direction: Direction::In,
            name: Cow::from("foo"),
            path: Cow::from("bar"),
            connection: None,
        };

        assert_eq!(
            to_string(&binding).unwrap(),
            r#"{"type":"blob","direction":"in","name":"foo","path":"bar"}"#
        );
    }

    #[test]
    fn it_parses_attribute_arguments() {
        let binding: Blob = (
            vec![
                parse_str::<NestedMeta>(r#"name = "foo""#).unwrap(),
                parse_str::<NestedMeta>(r#"path = "bar""#).unwrap(),
                parse_str::<NestedMeta>(r#"connection = "baz""#).unwrap(),
            ],
            Span::call_site(),
        )
            .into();

        assert_eq!(binding.direction, Direction::In);
        assert_eq!(binding.name.as_ref(), "foo");
        assert_eq!(binding.path.as_ref(), "bar");
        assert_eq!(binding.connection.unwrap().as_ref(), "baz");
    }

    #[test]
    fn it_requires_the_name_attribute_argument() {
        should_panic(
            || {
                let _: Blob = (
                    vec![parse_str::<NestedMeta>(r#"path = "foo""#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "the 'name' argument is required for this binding",
        );
    }

    #[test]
    fn it_requires_the_name_attribute_be_a_string() {
        should_panic(
            || {
                let _: Blob = (
                    vec![parse_str::<NestedMeta>(r#"name = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'name' argument",
        );
    }

    #[test]
    fn it_requires_the_path_attribute_argument() {
        should_panic(
            || {
                let _: Blob = (
                    vec![parse_str::<NestedMeta>(r#"name = "foo""#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "the 'path' argument is required for this binding",
        );
    }

    #[test]
    fn it_requires_the_path_attribute_be_a_string() {
        should_panic(
            || {
                let _: Blob = (
                    vec![parse_str::<NestedMeta>(r#"path = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'path' argument",
        );
    }

    #[test]
    fn it_requires_the_connection_attribute_be_a_string() {
        should_panic(
            || {
                let _: Blob = (
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
        let binding = Blob {
            direction: Direction::Out,
            name: Cow::from("foo"),
            path: Cow::from("bar"),
            connection: Some(Cow::from("baz")),
        };

        let mut stream = TokenStream::new();
        binding.to_tokens(&mut stream);
        let mut tokens = stream.to_string();
        tokens.retain(|c| c != ' ');

        assert_eq!(tokens, r#"::azure_functions::codegen::bindings::Blob{direction:::azure_functions::codegen::bindings::Direction::Out,name:::std::borrow::Cow::Borrowed("foo"),path:::std::borrow::Cow::Borrowed("bar"),connection:Some(::std::borrow::Cow::Borrowed("baz")),}"#);
    }
}
