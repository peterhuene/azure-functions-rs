use azure_functions_shared::codegen;
use proc_macro::{Diagnostic, Span, TokenStream};
use proc_macro2::Delimiter;
use quote::ToTokens;
use std::convert::TryFrom;
use syn::buffer::TokenBuffer;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::synom::Synom;
use syn::{parse, parse2, Attribute, Ident, Lit, LitStr, Path, PathSegment};

pub fn to_camel_case(input: &str) -> String {
    let mut result = String::new();
    let mut capitalize = false;
    let mut first = true;
    for ch in input.chars() {
        if ch == '_' {
            capitalize = true;
        } else {
            result.push(match capitalize && !first {
                true => ch.to_ascii_uppercase(),
                false => ch,
            });
            first = false;
            capitalize = false;
        }
    }
    result
}

pub struct AttributeArguments {
    pub span: Span,
    pub list: Vec<(Ident, Lit)>,
}

impl AttributeArguments {
    pub fn with_name(name: &str, span: ::proc_macro2::Span) -> Self {
        AttributeArguments {
            span: span.unstable(),
            list: vec![(
                Ident::new("name", span.clone()),
                Lit::Str(LitStr::new(name, span)),
            )],
        }
    }
}

impl TryFrom<TokenStream> for AttributeArguments {
    type Error = Diagnostic;

    fn try_from(stream: TokenStream) -> Result<Self, Self::Error> {
        if stream.is_empty() {
            return Ok(AttributeArguments {
                span: Span::call_site(),
                list: Vec::new(),
            });
        }

        parse::<AttributeArguments>(stream).map_err(|e| Span::call_site().error(e.to_string()))
    }
}

impl TryFrom<::proc_macro2::TokenStream> for AttributeArguments {
    type Error = Diagnostic;

    fn try_from(stream: ::proc_macro2::TokenStream) -> Result<Self, Self::Error> {
        if stream.is_empty() {
            return Ok(AttributeArguments {
                span: Span::call_site(),
                list: Vec::new(),
            });
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

        let mut args = AttributeArguments::try_from(stream)
            .map_err(|_| span.unstable().error("failed to parse attribute"))?;
        args.span = span.unstable();
        Ok(args)
    }
}

impl Synom for AttributeArguments {
    named!(parse -> Self, map!(
        Punctuated::<ArgumentAssignmentExpr, Token![,]>::parse_terminated_nonempty,
        |exprs| AttributeArguments{
            span: Span::call_site(),
            list: exprs.into_iter().fold(Vec::new(), |mut list, expr| {
                let ArgumentAssignmentExpr(name, value) = expr;
                list.push((name, value));
                list
            }),
        }));

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

pub struct QuotableBorrowedStr<'a>(pub &'a str);

impl ToTokens for QuotableBorrowedStr<'_> {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        let s = self.0;
        quote!(::std::borrow::Cow::Borrowed(#s)).to_tokens(tokens);
    }
}

pub struct QuotableOption<T>(pub Option<T>);

impl<T: ToTokens> ToTokens for QuotableOption<T> {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        match &self.0 {
            Some(inner) => quote!(Some(#inner)),
            None => quote!(None),
        }.to_tokens(tokens);
    }
}

pub struct QuotableDirection(pub codegen::Direction);

impl ToTokens for QuotableDirection {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        match self.0 {
            codegen::Direction::In => {
                quote!(::azure_functions::codegen::Direction::In).to_tokens(tokens)
            }
            codegen::Direction::InOut => {
                quote!(::azure_functions::codegen::Direction::InOut).to_tokens(tokens)
            }
            codegen::Direction::Out => {
                quote!(::azure_functions::codegen::Direction::Out).to_tokens(tokens);
            }
        };
    }
}

#[derive(Default)]
pub struct PathVec(Vec<Path>);

impl Synom for PathVec {
    named!(parse -> Self, map!(
        call!(Punctuated::<Path, Token![,]>::parse_terminated_nonempty),
        |paths| PathVec(paths.into_iter().collect())
    ));
}

impl IntoIterator for PathVec {
    type Item = Path;
    type IntoIter = ::std::vec::IntoIter<Path>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
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

pub fn path_to_string(path: &Path) -> String {
    let mut s = String::new();

    for segment in path.segments.iter() {
        if !s.is_empty() {
            s += "::";
        }

        s += &segment.ident.to_string();
    }

    s
}

pub fn last_segment_in_path(path: &Path) -> &PathSegment {
    path.segments
        .iter()
        .last()
        .expect("expected at least one segment in path")
}
