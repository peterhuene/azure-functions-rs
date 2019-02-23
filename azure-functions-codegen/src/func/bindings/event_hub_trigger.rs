use crate::util::{
    to_camel_case, AttributeArguments, MacroError, QuotableBorrowedStr, QuotableOption, TryFrom,
};
use azure_functions_shared::codegen;
use quote::{quote, ToTokens};
use std::borrow::Cow;
use syn::spanned::Spanned;
use syn::Lit;

pub struct EventHubTrigger<'a>(pub Cow<'a, codegen::bindings::EventHubTrigger>);

impl TryFrom<AttributeArguments> for EventHubTrigger<'_> {
    type Error = MacroError;

    fn try_from(args: AttributeArguments) -> Result<Self, Self::Error> {
        let mut name = None;
        let mut connection = None;
        let mut event_hub_name = None;
        let mut consumer_group = None;

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
                "connection" => match value {
                    Lit::Str(s) => {
                        connection = Some(Cow::Owned(s.value()));
                    }
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal string value for the 'connection' argument",
                        )
                            .into());
                    }
                },
                "event_hub_name" => match value {
                    Lit::Str(s) => {
                        event_hub_name = Some(Cow::Owned(s.value()));
                    }
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal string value for the 'event_hub_name' argument",
                        )
                            .into());
                    }
                },
                "consumer_group" => match value {
                    Lit::Str(s) => {
                        consumer_group = Some(Cow::Owned(s.value()));
                    }
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal string value for the 'consumer_group' argument",
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

        if connection.is_none() {
            return Err((
                args.span,
                "the 'connection' argument is required for Event Hub trigger bindings.",
            )
                .into());
        }

        Ok(EventHubTrigger(Cow::Owned(
            codegen::bindings::EventHubTrigger {
                name: name.expect("expected a name for a Event Hub trigger binding"),
                connection: connection.unwrap(),
                event_hub_name,
                consumer_group,
            },
        )))
    }
}

impl ToTokens for EventHubTrigger<'_> {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        let name = QuotableBorrowedStr(&self.0.name);
        let connection = QuotableBorrowedStr(&self.0.connection);
        let event_hub_name = QuotableOption(
            self.0
                .event_hub_name
                .as_ref()
                .map(|x| QuotableBorrowedStr(x)),
        );
        let consumer_group = QuotableOption(
            self.0
                .consumer_group
                .as_ref()
                .map(|x| QuotableBorrowedStr(x)),
        );

        quote!(::azure_functions::codegen::bindings::EventHubTrigger {
            name: #name,
            connection: #connection,
            event_hub_name: #event_hub_name,
            consumer_group: #consumer_group,
        })
        .to_tokens(tokens)
    }
}
