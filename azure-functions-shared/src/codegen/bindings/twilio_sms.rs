use azure_functions_shared_codegen::binding;
use std::borrow::Cow;

#[binding(name = "twilioSms", direction = "out")]
pub struct TwilioSms {
    #[field(camel_case_value = true)]
    pub name: Cow<'static, str>,
    #[field(name = "accountSidSetting")]
    pub account_sid: Option<Cow<'static, str>>,
    #[field(name = "authTokenSetting")]
    pub auth_token: Option<Cow<'static, str>>,
    pub from: Option<Cow<'static, str>>,
    pub body: Option<Cow<'static, str>>,
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
        let binding = TwilioSms {
            name: Cow::from("foo"),
            account_sid: Some(Cow::from("bar")),
            auth_token: Some(Cow::from("baz")),
            from: Some(Cow::from("jam")),
            body: Some(Cow::from("cake")),
        };

        assert_eq!(
            to_string(&binding).unwrap(),
            r#"{"type":"twilioSms","direction":"out","name":"foo","accountSidSetting":"bar","authTokenSetting":"baz","from":"jam","body":"cake"}"#
        );
    }

    #[test]
    fn it_parses_attribute_arguments() {
        let binding: TwilioSms = (
            vec![
                parse_str::<NestedMeta>(r#"name = "foo""#).unwrap(),
                parse_str::<NestedMeta>(r#"account_sid = "bar""#).unwrap(),
                parse_str::<NestedMeta>(r#"auth_token = "baz""#).unwrap(),
                parse_str::<NestedMeta>(r#"from = "jam""#).unwrap(),
                parse_str::<NestedMeta>(r#"body = "cake""#).unwrap(),
            ],
            Span::call_site(),
        )
            .into();

        assert_eq!(binding.name.as_ref(), "foo");
        assert_eq!(binding.account_sid.unwrap().as_ref(), "bar");
        assert_eq!(binding.auth_token.unwrap().as_ref(), "baz");
        assert_eq!(binding.from.unwrap().as_ref(), "jam");
        assert_eq!(binding.body.unwrap().as_ref(), "cake");
    }

    #[test]
    fn it_requires_the_name_attribute_argument() {
        should_panic(
            || {
                let _: TwilioSms = (vec![], Span::call_site()).into();
            },
            "the 'name' argument is required for this binding",
        );
    }

    #[test]
    fn it_requires_the_name_attribute_be_a_string() {
        should_panic(
            || {
                let _: TwilioSms = (
                    vec![parse_str::<NestedMeta>(r#"name = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'name' argument",
        );
    }

    #[test]
    fn it_requires_the_account_sid_attribute_be_a_string() {
        should_panic(
            || {
                let _: TwilioSms = (
                    vec![parse_str::<NestedMeta>(r#"account_sid = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'account_sid' argument",
        );
    }

    #[test]
    fn it_requires_the_auth_token_attribute_be_a_string() {
        should_panic(
            || {
                let _: TwilioSms = (
                    vec![parse_str::<NestedMeta>(r#"auth_token = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'auth_token' argument",
        );
    }

    #[test]
    fn it_requires_the_from_attribute_be_a_string() {
        should_panic(
            || {
                let _: TwilioSms = (
                    vec![parse_str::<NestedMeta>(r#"from = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'from' argument",
        );
    }

    #[test]
    fn it_requires_the_body_attribute_be_a_string() {
        should_panic(
            || {
                let _: TwilioSms = (
                    vec![parse_str::<NestedMeta>(r#"body = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'body' argument",
        );
    }

    #[test]
    fn it_converts_to_tokens() {
        let binding = TwilioSms {
            name: Cow::from("foo"),
            account_sid: Some(Cow::from("bar")),
            auth_token: Some(Cow::from("baz")),
            from: Some(Cow::from("jam")),
            body: Some(Cow::from("cake")),
        };

        let mut stream = TokenStream::new();
        binding.to_tokens(&mut stream);
        let mut tokens = stream.to_string();
        tokens.retain(|c| c != ' ');

        assert_eq!(tokens, r#"::azure_functions::codegen::bindings::TwilioSms{name:::std::borrow::Cow::Borrowed("foo"),account_sid:Some(::std::borrow::Cow::Borrowed("bar")),auth_token:Some(::std::borrow::Cow::Borrowed("baz")),from:Some(::std::borrow::Cow::Borrowed("jam")),body:Some(::std::borrow::Cow::Borrowed("cake")),}"#);
    }
}
