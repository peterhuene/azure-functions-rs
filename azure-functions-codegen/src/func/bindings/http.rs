use azure_functions_shared::codegen;
use proc_macro::Diagnostic;
use proc_macro2::TokenStream;
use quote::ToTokens;
use std::borrow::Cow;
use std::convert::TryFrom;
use syn::spanned::Spanned;
use syn::Lit;
use util::{to_camel_case, AttributeArguments, QuotableBorrowedStr};

pub struct Http<'a>(pub Cow<'a, codegen::bindings::Http>);

impl<'a> TryFrom<&'a AttributeArguments> for Http<'a> {
    type Error = Diagnostic;

    fn try_from(args: &AttributeArguments) -> Result<Self, Self::Error> {
        let mut name = None;

        for (key, value) in args.list.iter() {
            let key_str = key.to_string();

            match key_str.as_str() {
                "name" => match value {
                    Lit::Str(s) => {
                        name = Some(Cow::Owned(to_camel_case(&s.value())));
                    }
                    _ => {
                        return Err(value
                            .span()
                            .unstable()
                            .error("expected a literal string value for the 'name' argument"));
                    }
                },
                _ => {
                    return Err(key
                        .span()
                        .unstable()
                        .error(format!("unsupported attribute argument '{}'", key_str)));
                }
            };
        }

        Ok(Http(Cow::Owned(codegen::bindings::Http {
            name: name.expect("expected a name for the Http binding"),
        })))
    }
}

impl<'a> ToTokens for Http<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = QuotableBorrowedStr(&self.0.name);
        quote!(::azure_functions::codegen::bindings::Http { name: #name }).to_tokens(tokens)
    }
}
