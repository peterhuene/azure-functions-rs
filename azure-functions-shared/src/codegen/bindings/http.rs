use azure_functions_shared_codegen::binding;
use std::borrow::Cow;

#[binding(name = "http", direction = "out")]
pub struct Http {
    #[field(camel_case_value = true)]
    pub name: Cow<'static, str>,
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
        let binding = Http {
            name: Cow::from("foo"),
        };

        assert_eq!(
            to_string(&binding).unwrap(),
            r#"{"type":"http","direction":"out","name":"foo"}"#
        );
    }

    #[test]
    fn it_parses_attribute_arguments() {
        let binding: Http = (
            vec![parse_str::<NestedMeta>(r#"name = "foo""#).unwrap()],
            Span::call_site(),
        )
            .into();

        assert_eq!(binding.name.as_ref(), "foo");
    }

    #[test]
    fn it_requires_the_name_attribute_argument() {
        should_panic(
            || {
                let _: Http = (vec![], Span::call_site()).into();
            },
            "the 'name' argument is required for this binding",
        );
    }

    #[test]
    fn it_requires_the_name_attribute_be_a_string() {
        should_panic(
            || {
                let _: Http = (
                    vec![parse_str::<NestedMeta>(r#"name = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'name' argument",
        );
    }

    #[test]
    fn it_converts_to_tokens() {
        let binding = Http {
            name: Cow::from("foo"),
        };

        let mut stream = TokenStream::new();
        binding.to_tokens(&mut stream);
        let mut tokens = stream.to_string();
        tokens.retain(|c| c != ' ');

        assert_eq!(tokens, r#"::azure_functions::codegen::bindings::Http{name:::std::borrow::Cow::Borrowed("foo"),}"#);
    }
}
