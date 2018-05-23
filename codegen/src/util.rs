use proc_macro::{Diagnostic, Span, TokenStream};
use proc_macro2::Delimiter;
use quote::ToTokens;
use std::convert::TryFrom;
use syn::buffer::TokenBuffer;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::synom::Synom;
use syn::{parse, parse2, Attribute, Ident, Lit, LitStr, Path};

#[derive(Default)]
pub struct PathVec {
    paths: Vec<Path>,
}

impl Synom for PathVec {
    named!(parse -> Self, map!(
        call!(Punctuated::<Path, Token![,]>::parse_terminated_nonempty),
        |paths| PathVec {
            paths: paths.into_iter().collect(),
        }
    ));
}

impl IntoIterator for PathVec {
    type Item = Path;
    type IntoIter = ::std::vec::IntoIter<Path>;

    fn into_iter(self) -> Self::IntoIter {
        self.paths.into_iter()
    }
}

impl TryFrom<TokenStream> for PathVec {
    type Error = Diagnostic;

    fn try_from(stream: TokenStream) -> Result<Self, Self::Error> {
        if stream.is_empty() {
            return Ok(Self::default());
        }

        parse::<PathVec>(stream).map_err(|e| Span::call_site().error(e.to_string()))
    }
}

pub struct AttributeArguments(pub Vec<(Ident, Lit)>);

impl AttributeArguments {
    pub fn with_name(name: &str, span: ::proc_macro2::Span) -> Self {
        AttributeArguments(vec![(
            Ident::new("name", span.clone()),
            Lit::Str(LitStr::new(name, span)),
        )])
    }
}

impl TryFrom<TokenStream> for AttributeArguments {
    type Error = Diagnostic;

    fn try_from(stream: TokenStream) -> Result<Self, Self::Error> {
        if stream.is_empty() {
            return Ok(AttributeArguments(Vec::new()));
        }

        parse::<AttributeArguments>(stream).map_err(|e| Span::call_site().error(e.to_string()))
    }
}

impl TryFrom<::proc_macro2::TokenStream> for AttributeArguments {
    type Error = Diagnostic;

    fn try_from(stream: ::proc_macro2::TokenStream) -> Result<Self, Self::Error> {
        if stream.is_empty() {
            return Ok(AttributeArguments(Vec::new()));
        }

        parse2::<AttributeArguments>(stream).map_err(|e| Span::call_site().error(e.to_string()))
    }
}

impl TryFrom<Attribute> for AttributeArguments {
    type Error = Diagnostic;

    fn try_from(attr: Attribute) -> Result<Self, Self::Error> {
        let span = attr.span();
        let stream = match TokenBuffer::new2(attr.tts)
            .begin()
            .group(Delimiter::Parenthesis)
        {
            Some((tree, _, _)) => tree.token_stream(),
            None => {
                return Err(span.unstable().error("failed to parse attribute"));
            }
        };

        AttributeArguments::try_from(stream)
    }
}

impl Synom for AttributeArguments {
    named!(parse -> Self, map!(
        Punctuated::<ArgumentAssignmentExpr, Token![,]>::parse_terminated_nonempty,
        |exprs| AttributeArguments(exprs.into_iter().fold(Vec::new(), |mut list, expr| {
            let ArgumentAssignmentExpr(name, value) = expr;
            list.push((name, value));
            list
        })
    )));

    fn description() -> Option<&'static str> {
        Some("attribute arguments")
    }
}

struct ArgumentAssignmentExpr(Ident, Lit);

impl Synom for ArgumentAssignmentExpr {
    named!(parse -> Self, do_parse!(
        name: syn!(Ident) >>
        punct!(=) >>
        value: syn!(Lit) >>
        (ArgumentAssignmentExpr(name, value))
    ));

    fn description() -> Option<&'static str> {
        Some("attribute assignment expression")
    }
}

pub struct QuotableOption<T>(pub Option<T>);

impl<T: ToTokens> ToTokens for QuotableOption<T> {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        match self.0 {
            Some(ref t) => quote! { Some(#t) },
            None => quote! { None },
        }.to_tokens(tokens);
    }
}

pub trait ToString {
    fn to_string(&self) -> String;
}

impl ToString for Path {
    fn to_string(&self) -> String {
        let mut s = String::new();

        for segment in self.segments.iter() {
            if !s.is_empty() {
                s += "::";
            }

            s += &segment.ident.to_string();
        }

        s
    }
}
