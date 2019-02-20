use crate::util::{to_camel_case, AttributeArguments, MacroError, QuotableBorrowedStr, TryFrom};
use azure_functions_shared::codegen;
use quote::{quote, ToTokens};
use std::borrow::Cow;
use syn::spanned::Spanned;
use syn::Lit;

pub struct EventGridTrigger<'a>(pub Cow<'a, codegen::bindings::EventGridTrigger>);

impl TryFrom<AttributeArguments> for EventGridTrigger<'_> {
    type Error = MacroError;

    fn try_from(args: AttributeArguments) -> Result<Self, Self::Error> {
        let mut name = None;

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
                _ => {
                    return Err((
                        key.span(),
                        format!("unsupported binding attribute argument '{}'", key_str).as_ref(),
                    )
                        .into());
                }
            };
        }

        Ok(EventGridTrigger(Cow::Owned(
            codegen::bindings::EventGridTrigger {
                name: name.expect("expected a name for a Event Grid trigger binding"),
            },
        )))
    }
}

impl ToTokens for EventGridTrigger<'_> {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        let name = QuotableBorrowedStr(&self.0.name);

        quote!(::azure_functions::codegen::bindings::EventGridTrigger {
            name: #name,
        })
        .to_tokens(tokens)
    }
}
