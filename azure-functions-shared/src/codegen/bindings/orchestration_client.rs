use azure_functions_shared_codegen::binding;
use std::borrow::Cow;

#[binding(name = "orchestrationClient", direction = "in")]
pub struct OrchestrationClient {
    #[field(camel_case_value = true)]
    pub name: Cow<'static, str>,
    #[field(name = "taskHub")]
    pub task_hub: Option<Cow<'static, str>>,
    #[field(name = "connectionName")]
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
        let binding = OrchestrationClient {
            name: Cow::from("foo"),
            task_hub: Some(Cow::from("bar")),
            connection: Some(Cow::from("baz")),
        };

        assert_eq!(
            to_string(&binding).unwrap(),
            r#"{"type":"orchestrationClient","direction":"in","name":"foo","taskHub":"bar","connectionName":"baz"}"#
        );
    }

    #[test]
    fn it_parses_attribute_arguments() {
        let binding: OrchestrationClient = (
            vec![
                parse_str::<NestedMeta>(r#"name = "foo""#).unwrap(),
                parse_str::<NestedMeta>(r#"task_hub = "bar""#).unwrap(),
                parse_str::<NestedMeta>(r#"connection = "baz""#).unwrap(),
            ],
            Span::call_site(),
        )
            .into();

        assert_eq!(binding.name.as_ref(), "foo");
        assert_eq!(binding.task_hub.as_ref().unwrap(), "bar");
        assert_eq!(binding.connection.as_ref().unwrap(), "baz");
    }

    #[test]
    fn it_requires_the_name_attribute_argument() {
        should_panic(
            || {
                let _: OrchestrationClient = (vec![], Span::call_site()).into();
            },
            "the 'name' argument is required for this binding",
        );
    }

    #[test]
    fn it_requires_the_name_attribute_be_a_string() {
        should_panic(
            || {
                let _: OrchestrationClient = (
                    vec![parse_str::<NestedMeta>(r#"name = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'name' argument",
        );
    }

    #[test]
    fn it_requires_the_task_hub_attribute_be_a_string() {
        should_panic(
            || {
                let _: OrchestrationClient = (
                    vec![parse_str::<NestedMeta>(r#"task_hub = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'task_hub' argument",
        );
    }

    #[test]
    fn it_requires_the_connection_attribute_be_a_string() {
        should_panic(
            || {
                let _: OrchestrationClient = (
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
        let binding = OrchestrationClient {
            name: Cow::from("foo"),
            task_hub: Some(Cow::from("bar")),
            connection: Some(Cow::from("baz")),
        };

        let mut stream = TokenStream::new();
        binding.to_tokens(&mut stream);
        let mut tokens = stream.to_string();
        tokens.retain(|c| c != ' ');

        assert_eq!(
            tokens,
            r#"::azure_functions::codegen::bindings::OrchestrationClient{name:::std::borrow::Cow::Borrowed("foo"),task_hub:Some(::std::borrow::Cow::Borrowed("bar")),connection:Some(::std::borrow::Cow::Borrowed("baz")),}"#
        );
    }
}
