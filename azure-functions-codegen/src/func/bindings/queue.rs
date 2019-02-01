use crate::util::{
    to_camel_case, AttributeArguments, MacroError, QuotableBorrowedStr, QuotableOption, TryFrom,
};
use azure_functions_shared::codegen;
use quote::{quote, ToTokens};
use std::borrow::Cow;
use syn::spanned::Spanned;
use syn::Lit;

pub struct Queue<'a>(pub Cow<'a, codegen::bindings::Queue>);

impl TryFrom<AttributeArguments> for Queue<'_> {
    type Error = MacroError;

    fn try_from(args: AttributeArguments) -> Result<Self, Self::Error> {
        let mut name = None;
        let mut queue_name = None;
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
                "queue_name" => match value {
                    Lit::Str(s) => {
                        queue_name = Some(Cow::Owned(s.value()));
                    }
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal string value for the 'queue_name' argument",
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

        if queue_name.is_none() {
            return Err((
                args.span,
                "the 'queue_name' argument is required for queue message bindings.",
            )
                .into());
        }

        Ok(Queue(Cow::Owned(codegen::bindings::Queue {
            name: name.expect("expected a name for a queue binding"),
            queue_name: queue_name.expect("expected a queue name for a queue binding"),
            connection,
        })))
    }
}

impl ToTokens for Queue<'_> {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        let name = QuotableBorrowedStr(&self.0.name);
        let queue_name = QuotableBorrowedStr(&self.0.queue_name);
        let connection = QuotableOption(self.0.connection.as_ref().map(|x| QuotableBorrowedStr(x)));

        quote!(::azure_functions::codegen::bindings::Queue {
            name: #name,
            queue_name: #queue_name,
            connection: #connection,
        })
        .to_tokens(tokens)
    }
}
