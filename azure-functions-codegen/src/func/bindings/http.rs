use crate::util::{to_camel_case, AttributeArguments, MacroError, QuotableBorrowedStr, TryFrom};
use azure_functions_shared::codegen;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::borrow::Cow;
use syn::spanned::Spanned;
use syn::Lit;

pub struct Http<'a>(pub Cow<'a, codegen::bindings::Http>);

impl TryFrom<AttributeArguments> for Http<'_> {
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
                        format!("unsupported attribute argument '{}'", key_str).as_ref(),
                    )
                        .into());
                }
            };
        }

        Ok(Http(Cow::Owned(codegen::bindings::Http {
            name: name.expect("expected a name for a http binding"),
        })))
    }
}

impl ToTokens for Http<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = QuotableBorrowedStr(&self.0.name);
        quote!(::azure_functions::codegen::bindings::Http { name: #name }).to_tokens(tokens)
    }
}
