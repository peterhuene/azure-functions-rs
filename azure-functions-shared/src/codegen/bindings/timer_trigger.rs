use azure_functions_shared_codegen::binding;
use std::borrow::Cow;

#[binding(name = "timerTrigger", direction = "in")]
pub struct TimerTrigger {
    #[field(camel_case_value = true)]
    pub name: Cow<'static, str>,
    pub schedule: Cow<'static, str>,
    #[field(name = "runOnStartup")]
    pub run_on_startup: Option<bool>,
    #[field(name = "useMonitor")]
    pub use_monitor: Option<bool>,
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
        let binding = TimerTrigger {
            name: Cow::from("foo"),
            schedule: Cow::from("bar"),
            run_on_startup: Some(true),
            use_monitor: Some(false),
        };

        assert_eq!(
            to_string(&binding).unwrap(),
            r#"{"type":"timerTrigger","direction":"in","name":"foo","schedule":"bar","runOnStartup":true,"useMonitor":false}"#
        );
    }

    #[test]
    fn it_parses_attribute_arguments() {
        let binding: TimerTrigger = (
            vec![
                parse_str::<NestedMeta>(r#"name = "foo""#).unwrap(),
                parse_str::<NestedMeta>(r#"schedule = "bar""#).unwrap(),
                parse_str::<NestedMeta>(r#"run_on_startup = true"#).unwrap(),
                parse_str::<NestedMeta>(r#"use_monitor = false"#).unwrap(),
            ],
            Span::call_site(),
        )
            .into();

        assert_eq!(binding.name.as_ref(), "foo");
        assert_eq!(binding.schedule.as_ref(), "bar");
        assert_eq!(binding.run_on_startup.unwrap(), true);
        assert_eq!(binding.use_monitor.unwrap(), false);
    }

    #[test]
    fn it_requires_the_name_attribute_argument() {
        should_panic(
            || {
                let _: TimerTrigger = (vec![], Span::call_site()).into();
            },
            "the 'name' argument is required for this binding",
        );
    }

    #[test]
    fn it_requires_the_name_attribute_be_a_string() {
        should_panic(
            || {
                let _: TimerTrigger = (
                    vec![parse_str::<NestedMeta>(r#"name = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'name' argument",
        );
    }

    #[test]
    fn it_requires_the_schedule_attribute_argument() {
        should_panic(
            || {
                let _: TimerTrigger = (
                    vec![parse_str::<NestedMeta>(r#"name = "foo""#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "the 'schedule' argument is required for this binding",
        );
    }

    #[test]
    fn it_requires_the_schedule_attribute_be_a_string() {
        should_panic(
            || {
                let _: TimerTrigger = (
                    vec![parse_str::<NestedMeta>(r#"schedule = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'schedule' argument",
        );
    }

    #[test]
    fn it_requires_the_run_on_startup_attribute_be_a_bool() {
        should_panic(
            || {
                let _: TimerTrigger = (
                    vec![parse_str::<NestedMeta>(r#"run_on_startup = 1"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal boolean value for the 'run_on_startup' argument",
        );
    }

    #[test]
    fn it_requires_the_use_monitor_attribute_be_a_bool() {
        should_panic(
            || {
                let _: TimerTrigger = (
                    vec![parse_str::<NestedMeta>(r#"use_monitor = 1"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal boolean value for the 'use_monitor' argument",
        );
    }

    #[test]
    fn it_converts_to_tokens() {
        let binding = TimerTrigger {
            name: Cow::from("foo"),
            schedule: Cow::from("bar"),
            run_on_startup: Some(true),
            use_monitor: Some(false),
        };

        let mut stream = TokenStream::new();
        binding.to_tokens(&mut stream);
        let mut tokens = stream.to_string();
        tokens.retain(|c| c != ' ');

        assert_eq!(tokens, r#"::azure_functions::codegen::bindings::TimerTrigger{name:::std::borrow::Cow::Borrowed("foo"),schedule:::std::borrow::Cow::Borrowed("bar"),run_on_startup:Some(true),use_monitor:Some(false),}"#);
    }
}
