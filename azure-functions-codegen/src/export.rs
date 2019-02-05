use crate::util::{PathVec, TryFrom};
use proc_macro::TokenStream;
use quote::quote;
use std::fmt::Write;
use syn::{parse_str, Expr};

pub fn attr_impl(input: TokenStream) -> TokenStream {
    let funcs = match PathVec::try_from(input.clone()) {
        Ok(funcs) => funcs,
        Err(e) => {
            e.emit();
            return input;
        }
    };
    let funcs: Vec<Expr> = funcs
        .into_iter()
        .map(|path| {
            let mut expr = String::new();
            if path.leading_colon.is_some() {
                expr += "::";
            }

            let mut segments = path.segments.into_iter().peekable();
            while let Some(segment) = segments.next() {
                if segments.peek().is_some() {
                    write!(&mut expr, "{}::", segment.ident).unwrap();
                } else {
                    write!(
                        &mut expr,
                        "__{}_FUNCTION",
                        segment.ident.to_string().to_uppercase()
                    )
                    .unwrap();
                }
            }

            parse_str::<Expr>(&expr).unwrap()
        })
        .collect();

    let expanded = quote! {
        &[#(&#funcs),*]
    };

    expanded.into()
}
