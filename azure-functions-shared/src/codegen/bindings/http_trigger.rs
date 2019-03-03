use azure_functions_shared_codegen::binding;
use std::borrow::Cow;

#[binding(name = "httpTrigger", direction = "in")]
pub struct HttpTrigger {
    #[field(camel_case_value = true)]
    pub name: Cow<'static, str>,
    #[field(name = "authLevel", values = "anonymous|function|admin")]
    pub auth_level: Option<Cow<'static, str>>,
    #[field(values = "get|post|delete|head|patch|put|options|trace")]
    pub methods: Cow<'static, [Cow<'static, str>]>,
    pub route: Option<Cow<'static, str>>,
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
        let binding = HttpTrigger {
            name: Cow::from("foo"),
            auth_level: Some(Cow::from("bar")),
            methods: Cow::from(vec![Cow::from("foo"), Cow::from("bar"), Cow::from("baz")]),
            route: Some(Cow::from("baz")),
        };

        assert_eq!(
            to_string(&binding).unwrap(),
            r#"{"type":"httpTrigger","direction":"in","name":"foo","authLevel":"bar","methods":["foo","bar","baz"],"route":"baz"}"#
        );
    }

    #[test]
    fn it_parses_attribute_arguments() {
        let binding: HttpTrigger = (
            vec![
                parse_str::<NestedMeta>(r#"name = "foo""#).unwrap(),
                parse_str::<NestedMeta>(r#"auth_level = "anonymous""#).unwrap(),
                parse_str::<NestedMeta>(r#"methods = "get|put""#).unwrap(),
                parse_str::<NestedMeta>(r#"route = "/foo/bar/baz""#).unwrap(),
            ],
            Span::call_site(),
        )
            .into();

        assert_eq!(binding.name.as_ref(), "foo");
        assert_eq!(binding.auth_level.unwrap().as_ref(), "anonymous");
        assert_eq!(binding.methods.as_ref(), ["get", "put"]);
        assert_eq!(binding.route.unwrap().as_ref(), "/foo/bar/baz");
    }

    #[test]
    fn it_requires_the_name_attribute_argument() {
        should_panic(
            || {
                let _: HttpTrigger = (vec![], Span::call_site()).into();
            },
            "the 'name' argument is required for this binding",
        );
    }

    #[test]
    fn it_requires_the_name_attribute_be_a_string() {
        should_panic(
            || {
                let _: HttpTrigger = (
                    vec![parse_str::<NestedMeta>(r#"name = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'name' argument",
        );
    }

    #[test]
    fn it_requires_the_auth_level_attribute_be_a_string() {
        should_panic(
            || {
                let _: HttpTrigger = (
                    vec![parse_str::<NestedMeta>(r#"auth_level = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'auth_level' argument",
        );
    }

    #[test]
    fn it_accepts_valid_auth_levels() {
        let _: HttpTrigger = (
            vec![
                parse_str::<NestedMeta>(r#"name = "foo""#).unwrap(),
                parse_str::<NestedMeta>(r#"auth_level = "anonymous""#).unwrap(),
            ],
            Span::call_site(),
        )
            .into();

        let _: HttpTrigger = (
            vec![
                parse_str::<NestedMeta>(r#"name = "foo""#).unwrap(),
                parse_str::<NestedMeta>(r#"auth_level = "function""#).unwrap(),
            ],
            Span::call_site(),
        )
            .into();

        let _: HttpTrigger = (
            vec![
                parse_str::<NestedMeta>(r#"name = "foo""#).unwrap(),
                parse_str::<NestedMeta>(r#"auth_level = "admin""#).unwrap(),
            ],
            Span::call_site(),
        )
            .into();
    }

    #[test]
    fn it_rejects_invalid_auth_levels() {
        should_panic(
            || {
                let _: HttpTrigger = (
                    vec![parse_str::<NestedMeta>(r#"auth_level = "foo""#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "'foo' is not a valid value for the 'auth_level' attribute",
        );
    }

    #[test]
    fn it_requires_the_methods_attribute_be_a_string() {
        should_panic(
            || {
                let _: HttpTrigger = (
                    vec![parse_str::<NestedMeta>(r#"methods = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'methods' argument",
        );
    }

    #[test]
    fn it_accepts_valid_methods() {
        let _: HttpTrigger = (
            vec![
                parse_str::<NestedMeta>(r#"name = "foo""#).unwrap(),
                parse_str::<NestedMeta>(
                    r#"methods = "get|post|delete|head|patch|put|options|trace""#,
                )
                .unwrap(),
            ],
            Span::call_site(),
        )
            .into();
    }

    #[test]
    fn it_rejects_invalid_methods() {
        should_panic(
            || {
                let _: HttpTrigger = (
                    vec![parse_str::<NestedMeta>(r#"methods = "get|foo|post""#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "'foo' is not a valid value for the 'methods' attribute",
        );
    }

    #[test]
    fn it_requires_the_route_attribute_be_a_string() {
        should_panic(
            || {
                let _: HttpTrigger = (
                    vec![parse_str::<NestedMeta>(r#"route = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'route' argument",
        );
    }

    #[test]
    fn it_converts_to_tokens() {
        let binding = HttpTrigger {
            name: Cow::from("foo"),
            auth_level: Some(Cow::from("bar")),
            methods: Cow::from(vec![Cow::from("foo"), Cow::from("bar"), Cow::from("baz")]),
            route: Some(Cow::from("baz")),
        };

        let mut stream = TokenStream::new();
        binding.to_tokens(&mut stream);
        let mut tokens = stream.to_string();
        tokens.retain(|c| c != ' ');

        assert_eq!(tokens, r#"::azure_functions::codegen::bindings::HttpTrigger{name:::std::borrow::Cow::Borrowed("foo"),auth_level:Some(::std::borrow::Cow::Borrowed("bar")),methods:::std::borrow::Cow::Borrowed(&[::std::borrow::Cow::Borrowed("foo"),::std::borrow::Cow::Borrowed("bar"),::std::borrow::Cow::Borrowed("baz"),]),route:Some(::std::borrow::Cow::Borrowed("baz")),}"#);
    }
}
