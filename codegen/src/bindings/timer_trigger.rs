use proc_macro::Diagnostic;
use quote::ToTokens;
use std::convert::TryFrom;
use syn::spanned::Spanned;
use syn::Lit;
use util::{to_camel_case, AttributeArguments, QuotableOption};

#[derive(Debug)]
pub struct TimerTrigger {
    pub name: String,
    pub schedule: Option<String>,
    pub run_on_startup: Option<bool>,
    pub use_monitor: Option<bool>,
}

impl<'a> TryFrom<&'a AttributeArguments> for TimerTrigger {
    type Error = Diagnostic;

    fn try_from(args: &'a AttributeArguments) -> Result<Self, Self::Error> {
        let mut name = None;
        let mut schedule = None;
        let mut run_on_startup = None;
        let mut use_monitor = None;

        for (key, value) in args.0.iter() {
            let key_str = key.to_string();

            match key_str.as_str() {
                "name" => match value {
                    Lit::Str(s) => {
                        name = Some(to_camel_case(&s.value()));
                    }
                    _ => {
                        return Err(value
                            .span()
                            .unstable()
                            .error("expected a literal string value for the 'name' argument"));
                    }
                },
                "schedule" => match value {
                    Lit::Str(s) => {
                        schedule = Some(s.value());
                    }
                    _ => {
                        return Err(value
                            .span()
                            .unstable()
                            .error("expected a literal string value for the 'schedule' argument"));
                    }
                },
                "run_on_startup" => match value {
                    Lit::Bool(b) => {
                        run_on_startup = Some(b.value);
                    }
                    _ => {
                        return Err(value.span().unstable().error(
                            "expected a literal boolean value for the 'run_on_startup' argument",
                        ));
                    }
                },
                "use_monitor" => match value {
                    Lit::Bool(b) => {
                        use_monitor = Some(b.value);
                    }
                    _ => {
                        return Err(value.span().unstable().error(
                            "expected a literal boolean value for the 'use_monitor' argument",
                        ));
                    }
                },
                _ => {
                    return Err(key.span().unstable().error(format!(
                        "unsupported binding attribute argument '{}'",
                        key_str
                    )));
                }
            };
        }

        Ok(TimerTrigger {
            name: name.expect("expected a name for the TimerTrigger binding"),
            schedule: schedule,
            run_on_startup: run_on_startup,
            use_monitor: use_monitor,
        })
    }
}

impl ToTokens for TimerTrigger {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        let name = &self.name;
        let schedule = QuotableOption(self.schedule.clone());
        let run_on_startup = QuotableOption(self.run_on_startup);
        let use_monitor = QuotableOption(self.use_monitor);

        quote!(::azure_functions::codegen::bindings::TimerTrigger {
            name: #name,
            schedule: &#schedule,
            run_on_startup: &#run_on_startup,
            use_monitor: &#use_monitor
        }).to_tokens(tokens)
    }
}
