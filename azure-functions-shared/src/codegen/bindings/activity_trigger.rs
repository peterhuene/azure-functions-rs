use azure_functions_shared_codegen::binding;
use std::borrow::Cow;

#[binding(name = "activityTrigger", direction = "in")]
pub struct ActivityTrigger {
    #[field(camel_case_value = true)]
    pub name: Cow<'static, str>,
    pub activity: Option<Cow<'static, str>>,
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
        let binding = ActivityTrigger {
            name: Cow::from("foo"),
            activity: Some(Cow::from("bar")),
        };

        assert_eq!(
            to_string(&binding).unwrap(),
            r#"{"type":"activityTrigger","direction":"in","name":"foo","activity":"bar"}"#
        );
    }

    #[test]
    fn it_parses_attribute_arguments() {
        let binding: ActivityTrigger = (
            vec![
                parse_str::<NestedMeta>(r#"name = "foo""#).unwrap(),
                parse_str::<NestedMeta>(r#"activity = "bar""#).unwrap(),
            ],
            Span::call_site(),
        )
            .into();

        assert_eq!(binding.name.as_ref(), "foo");
        assert_eq!(binding.activity.as_ref().unwrap(), "bar");
    }

    #[test]
    fn it_requires_the_name_attribute_argument() {
        should_panic(
            || {
                let _: ActivityTrigger = (vec![], Span::call_site()).into();
            },
            "the 'name' argument is required for this binding",
        );
    }

    #[test]
    fn it_requires_the_name_attribute_be_a_string() {
        should_panic(
            || {
                let _: ActivityTrigger = (
                    vec![parse_str::<NestedMeta>(r#"name = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'name' argument",
        );
    }

    #[test]
    fn it_requires_the_activity_attribute_be_a_string() {
        should_panic(
            || {
                let _: ActivityTrigger = (
                    vec![parse_str::<NestedMeta>(r#"activity = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'activity' argument",
        );
    }

    #[test]
    fn it_converts_to_tokens() {
        let binding = ActivityTrigger {
            name: Cow::from("foo"),
            activity: Some(Cow::from("bar")),
        };

        let mut stream = TokenStream::new();
        binding.to_tokens(&mut stream);
        let mut tokens = stream.to_string();
        tokens.retain(|c| c != ' ');

        assert_eq!(
            tokens,
            r#"::azure_functions::codegen::bindings::ActivityTrigger{name:::std::borrow::Cow::Borrowed("foo"),activity:Some(::std::borrow::Cow::Borrowed("bar")),}"#
        );
    }
}
