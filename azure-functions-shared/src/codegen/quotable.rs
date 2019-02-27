use crate::codegen::bindings::Direction;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub struct QuotableBorrowedStr<'a>(pub &'a str);

impl ToTokens for QuotableBorrowedStr<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let s = self.0;
        quote!(::std::borrow::Cow::Borrowed(#s)).to_tokens(tokens);
    }
}

pub struct QuotableOption<T>(pub Option<T>);

impl<T: ToTokens> ToTokens for QuotableOption<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match &self.0 {
            Some(inner) => quote!(Some(#inner)),
            None => quote!(None),
        }
        .to_tokens(tokens);
    }
}

pub struct QuotableDirection(pub Direction);

impl ToTokens for QuotableDirection {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.0 {
            Direction::In => {
                quote!(::azure_functions::codegen::bindings::Direction::In).to_tokens(tokens)
            }
            Direction::InOut => {
                quote!(::azure_functions::codegen::bindings::Direction::InOut).to_tokens(tokens)
            }
            Direction::Out => {
                quote!(::azure_functions::codegen::bindings::Direction::Out).to_tokens(tokens);
            }
        };
    }
}
