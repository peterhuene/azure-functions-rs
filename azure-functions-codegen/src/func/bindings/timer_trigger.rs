use crate::util::{
    to_camel_case, AttributeArguments, MacroError, QuotableBorrowedStr, QuotableOption, TryFrom,
};
use azure_functions_shared::codegen;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::borrow::Cow;
use syn::spanned::Spanned;
use syn::Lit;

pub struct TimerTrigger<'a>(pub Cow<'a, codegen::bindings::TimerTrigger>);

impl TryFrom<AttributeArguments> for TimerTrigger<'_> {
    type Error = MacroError;

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
                            "expected a literal string value for the 'name' argument",
                        )
                            .into());
                    }
                },
                "schedule" => match value {
                    Lit::Str(s) => {
                        schedule = Some(Cow::Owned(s.value()));
                    }
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal string value for the 'schedule' argument",
                        )
                            .into());
                    }
                },
                "run_on_startup" => {
                    match value {
                        Lit::Bool(b) => {
                            run_on_startup = Some(b.value);
                        }
                        _ => {
                            return Err((value.span(),
                            "expected a literal boolean value for the 'run_on_startup' argument",
                        ).into());
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
                            "expected a literal boolean value for the 'use_monitor' argument",
                        )
                            .into());
                    }
                },
                _ => {
                    return Err((
                        key.span(),
                        format!("unsupported binding attribute argument '{}'", key_str).as_ref(),
                    )
                        .into());
                }
            };
        }

        Ok(TimerTrigger(Cow::Owned(codegen::bindings::TimerTrigger {
            name: name.expect("expected a name for a timer trigger binding"),
            schedule,
            run_on_startup,
            use_monitor,
        })))
    }
}

impl ToTokens for TimerTrigger<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = QuotableBorrowedStr(&self.0.name);
        let schedule = QuotableOption(self.0.schedule.as_ref().map(|x| QuotableBorrowedStr(x)));
        let run_on_startup = QuotableOption(self.0.run_on_startup);
        let use_monitor = QuotableOption(self.0.use_monitor);

        quote!(::azure_functions::codegen::bindings::TimerTrigger {
            name: #name,
            schedule: #schedule,
            run_on_startup: #run_on_startup,
            use_monitor: #use_monitor
        })
        .to_tokens(tokens)
    }
}
