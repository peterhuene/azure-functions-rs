use crate::func::Binding;
use crate::util::{AttributeArguments, QuotableBorrowedStr, QuotableOption};
use crate::util::{MacroError, TryFrom};
use azure_functions_shared::codegen;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use std::borrow::Cow;
use syn::spanned::Spanned;
use syn::{Ident, Lit};

pub struct Function<'a>(pub Cow<'a, codegen::Function>);

impl TryFrom<TokenStream> for Function<'_> {
    type Error = MacroError;

    fn try_from(stream: TokenStream) -> Result<Self, Self::Error> {
        let mut name = None;
        let mut disabled = None;

        for (key, value) in AttributeArguments::try_from(stream)?.list.into_iter() {
            let key_str = key.to_string();

            match key_str.as_str() {
                "name" => match &value {
                    Lit::Str(s) => {
                        name = s
                            .parse::<Ident>()
                            .map(|x| Some(Cow::Owned(x.to_string())))
                            .map_err(|_| {
                                (value.span(),
                                "a legal function identifier is required for the 'name' argument",
                            ).into()
                            })?;
                    }
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal string value for the 'name' argument",
                        )
                            .into());
                    }
                },
                "disabled" => match value {
                    Lit::Bool(b) => disabled = Some(b.value),
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal boolean value for the 'disabled' argument",
                        )
                            .into());
                    }
                },
                _ => {
                    return Err((
                        key.span(),
                        format!("unsupported argument '{}'", key_str).as_ref(),
                    )
                        .into());
                }
            };
        }

        Ok(Function(Cow::Owned(codegen::Function {
            name: name.unwrap_or(Cow::Borrowed("")),
            disabled: disabled.unwrap_or(false),
            bindings: Cow::Owned(Vec::new()),
            invoker_name: None,
            invoker: None,
            manifest_dir: None,
            file: None,
        })))
    }
}

impl ToTokens for Function<'_> {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        let name = QuotableBorrowedStr(&self.0.name);
        let disabled = self.0.disabled;
        let bindings = self.0.bindings.iter().filter_map(|x| {
            if x.is_context() {
                None
            } else {
                Some(Binding(x))
            }
        });
        let invoker_name =
            QuotableOption(self.0.invoker_name.as_ref().map(|x| QuotableBorrowedStr(x)));
        let invoker = Ident::new(
            self.0
                .invoker_name
                .as_ref()
                .expect("function must have an invoker"),
            Span::call_site(),
        );

        quote!(
        ::azure_functions::codegen::Function {
            name: #name,
            disabled: #disabled,
            bindings: ::std::borrow::Cow::Borrowed(&[#(#bindings),*]),
            invoker_name: #invoker_name,
            invoker: Some(#invoker),
            manifest_dir: Some(::std::borrow::Cow::Borrowed(env!("CARGO_MANIFEST_DIR"))),
            file: Some(::std::borrow::Cow::Borrowed(file!())),
        }
        )
        .to_tokens(tokens)
    }
}
