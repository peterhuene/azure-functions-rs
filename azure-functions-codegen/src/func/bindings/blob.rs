use crate::util::{
    to_camel_case, AttributeArguments, MacroError, QuotableBorrowedStr, QuotableDirection,
    QuotableOption, TryFrom,
};
use azure_functions_shared::codegen;
use quote::{quote, ToTokens};
use std::borrow::Cow;
use syn::spanned::Spanned;
use syn::Lit;

pub struct Blob<'a>(pub Cow<'a, codegen::bindings::Blob>);

impl TryFrom<AttributeArguments> for Blob<'_> {
    type Error = MacroError;

    fn try_from(args: AttributeArguments) -> Result<Self, Self::Error> {
        let mut name = None;
        let mut path = None;
        let mut connection = None;

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
                "path" => match value {
                    Lit::Str(s) => {
                        path = Some(Cow::Owned(s.value()));
                    }
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal string value for the 'path' argument",
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
                _ => {
                    return Err((
                        key.span(),
                        format!("unsupported binding attribute argument '{}'", key_str).as_ref(),
                    )
                        .into());
                }
            };
        }

        if path.is_none() {
            return Err((
                args.span,
                "the 'path' argument is required for blob bindings.",
            )
                .into());
        }

        Ok(Blob(Cow::Owned(codegen::bindings::Blob {
            name: name.expect("expected a name for a blob binding"),
            path: path.expect("expected a path for a blob binding"),
            connection,
            direction: codegen::Direction::In,
        })))
    }
}

impl ToTokens for Blob<'_> {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        let name = QuotableBorrowedStr(&self.0.name);
        let path = QuotableBorrowedStr(&self.0.path);
        let connection = QuotableOption(self.0.connection.as_ref().map(|x| QuotableBorrowedStr(x)));
        let direction = QuotableDirection(self.0.direction.clone());

        quote!(::azure_functions::codegen::bindings::Blob {
            name: #name,
            path: #path,
            connection: #connection,
            direction: #direction,
        })
        .to_tokens(tokens)
    }
}
