use azure_functions_shared_codegen::binding;
use std::borrow::Cow;

#[binding(name = "sendGrid", direction = "out")]
pub struct SendGrid {
    #[field(camel_case_value = true)]
    pub name: Cow<'static, str>,
    #[field(name = "apiKey")]
    pub api_key: Option<Cow<'static, str>>,
    pub to: Option<Cow<'static, str>>,
    pub from: Option<Cow<'static, str>>,
    pub subject: Option<Cow<'static, str>>,
    pub text: Option<Cow<'static, str>>,
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
        let binding = SendGrid {
            name: Cow::from("foo"),
            api_key: Some(Cow::from("bar")),
            to: Some(Cow::from("baz")),
            from: Some(Cow::from("jam")),
            subject: Some(Cow::from("cake")),
            text: Some(Cow::from("lie")),
        };

        assert_eq!(
            to_string(&binding).unwrap(),
            r#"{"type":"sendGrid","direction":"out","name":"foo","apiKey":"bar","to":"baz","from":"jam","subject":"cake","text":"lie"}"#
        );
    }

    #[test]
    fn it_parses_attribute_arguments() {
        let binding: SendGrid = (
            vec![
                parse_str::<NestedMeta>(r#"name = "foo""#).unwrap(),
                parse_str::<NestedMeta>(r#"api_key = "bar""#).unwrap(),
                parse_str::<NestedMeta>(r#"to = "baz""#).unwrap(),
                parse_str::<NestedMeta>(r#"from = "jam""#).unwrap(),
                parse_str::<NestedMeta>(r#"subject = "cake""#).unwrap(),
                parse_str::<NestedMeta>(r#"text = "lie""#).unwrap(),
            ],
            Span::call_site(),
        )
            .into();

        assert_eq!(binding.name.as_ref(), "foo");
        assert_eq!(binding.api_key.unwrap().as_ref(), "bar");
        assert_eq!(binding.to.unwrap().as_ref(), "baz");
        assert_eq!(binding.from.unwrap().as_ref(), "jam");
        assert_eq!(binding.subject.unwrap().as_ref(), "cake");
        assert_eq!(binding.text.unwrap().as_ref(), "lie");
    }

    #[test]
    fn it_requires_the_name_attribute_argument() {
        should_panic(
            || {
                let _: SendGrid = (vec![], Span::call_site()).into();
            },
            "the 'name' argument is required for this binding",
        );
    }

    #[test]
    fn it_requires_the_name_attribute_be_a_string() {
        should_panic(
            || {
                let _: SendGrid = (
                    vec![parse_str::<NestedMeta>(r#"name = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'name' argument",
        );
    }

    #[test]
    fn it_requires_the_api_key_attribute_be_a_string() {
        should_panic(
            || {
                let _: SendGrid = (
                    vec![parse_str::<NestedMeta>(r#"api_key = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'api_key' argument",
        );
    }

    #[test]
    fn it_requires_the_to_attribute_be_a_string() {
        should_panic(
            || {
                let _: SendGrid = (
                    vec![parse_str::<NestedMeta>(r#"to = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'to' argument",
        );
    }

    #[test]
    fn it_requires_the_from_attribute_be_a_string() {
        should_panic(
            || {
                let _: SendGrid = (
                    vec![parse_str::<NestedMeta>(r#"from = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'from' argument",
        );
    }

    #[test]
    fn it_requires_the_subject_attribute_be_a_string() {
        should_panic(
            || {
                let _: SendGrid = (
                    vec![parse_str::<NestedMeta>(r#"subject = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'subject' argument",
        );
    }

    #[test]
    fn it_requires_the_text_attribute_be_a_string() {
        should_panic(
            || {
                let _: SendGrid = (
                    vec![parse_str::<NestedMeta>(r#"text = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'text' argument",
        );
    }

    #[test]
    fn it_converts_to_tokens() {
        let binding = SendGrid {
            name: Cow::from("foo"),
            api_key: Some(Cow::from("bar")),
            to: Some(Cow::from("baz")),
            from: Some(Cow::from("jam")),
            subject: Some(Cow::from("cake")),
            text: Some(Cow::from("lie")),
        };

        let mut stream = TokenStream::new();
        binding.to_tokens(&mut stream);
        let mut tokens = stream.to_string();
        tokens.retain(|c| c != ' ');

        assert_eq!(tokens, r#"::azure_functions::codegen::bindings::SendGrid{name:::std::borrow::Cow::Borrowed("foo"),api_key:Some(::std::borrow::Cow::Borrowed("bar")),to:Some(::std::borrow::Cow::Borrowed("baz")),from:Some(::std::borrow::Cow::Borrowed("jam")),subject:Some(::std::borrow::Cow::Borrowed("cake")),text:Some(::std::borrow::Cow::Borrowed("lie")),}"#);
    }
}
