use crate::codegen::{
    quotable::{QuotableBorrowedStr, QuotableOption},
    AttributeArguments, TryFrom,
};
use crate::util::to_camel_case;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use serde::{ser::SerializeMap, Serialize, Serializer};
use std::borrow::Cow;
use syn::{spanned::Spanned, Lit};

pub const TIMER_TRIGGER_TYPE: &str = "timerTrigger";

#[derive(Debug, Clone)]
pub struct TimerTrigger {
    pub name: Cow<'static, str>,
    pub schedule: Option<Cow<'static, str>>,
    pub run_on_startup: Option<bool>,
    pub use_monitor: Option<bool>,
}

// TODO: when https://github.com/serde-rs/serde/issues/760 is resolved, remove implementation in favor of custom Serialize derive
// The fix would allow us to set the constant `type` and `direction` entries rather than having to emit them manually.
impl Serialize for TimerTrigger {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("type", TIMER_TRIGGER_TYPE)?;
        map.serialize_entry("direction", "in")?;

        if let Some(schedule) = self.schedule.as_ref() {
            map.serialize_entry("schedule", schedule)?;
        }
        if let Some(run_on_startup) = self.run_on_startup.as_ref() {
            map.serialize_entry("runOnStartup", run_on_startup)?;
        }
        if let Some(use_monitor) = self.use_monitor.as_ref() {
            map.serialize_entry("useMonitor", use_monitor)?;
        }

        map.end()
    }
}

impl TryFrom<AttributeArguments> for TimerTrigger {
    type Error = (Span, String);

    fn try_from(args: AttributeArguments) -> Result<Self, Self::Error> {
        let mut name = None;
        let mut schedule = None;
        let mut run_on_startup = None;
        let mut use_monitor = None;

        for (key, value) in args.list.iter() {
            let key_str = key.to_string();

            match key_str.as_str() {
                "name" => match value {
                    Lit::Str(s) => {
                        name = Some(Cow::Owned(to_camel_case(&s.value())));
                    }
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal string value for the 'name' argument".to_string(),
                        ));
                    }
                },
                "schedule" => match value {
                    Lit::Str(s) => {
                        schedule = Some(Cow::Owned(s.value()));
                    }
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal string value for the 'schedule' argument"
                                .to_string(),
                        ));
                    }
                },
                "run_on_startup" => {
                    match value {
                        Lit::Bool(b) => {
                            run_on_startup = Some(b.value);
                        }
                        _ => {
                            return Err((value.span(),
                            "expected a literal boolean value for the 'run_on_startup' argument".to_string(),
                        ));
                        }
                    }
                }
                "use_monitor" => match value {
                    Lit::Bool(b) => {
                        use_monitor = Some(b.value);
                    }
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal boolean value for the 'use_monitor' argument"
                                .to_string(),
                        ));
                    }
                },
                _ => {
                    return Err((
                        key.span(),
                        format!("unsupported binding attribute argument '{}'", key_str),
                    ));
                }
            };
        }

        Ok(TimerTrigger {
            name: name.unwrap(),
            schedule,
            run_on_startup,
            use_monitor,
        })
    }
}

impl ToTokens for TimerTrigger {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = QuotableBorrowedStr(&self.name);
        let schedule = QuotableOption(self.schedule.as_ref().map(|x| QuotableBorrowedStr(x)));
        let run_on_startup = QuotableOption(self.run_on_startup);
        let use_monitor = QuotableOption(self.use_monitor);

        quote!(::azure_functions::codegen::bindings::TimerTrigger {
            name: #name,
            schedule: #schedule,
            run_on_startup: #run_on_startup,
            use_monitor: #use_monitor
        })
        .to_tokens(tokens)
    }
}
