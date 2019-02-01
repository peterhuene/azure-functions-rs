use azure_functions_shared::codegen;
#[cfg(feature = "unstable")]
use proc_macro::Diagnostic;
use proc_macro::{Span, TokenStream};
use proc_macro2::{Delimiter, Span as Span2};
use quote::quote;
use quote::ToTokens;
use syn::buffer::TokenBuffer;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse, parse2, Attribute, Ident, Lit, LitStr, Path, PathSegment, Token};

pub trait TryFrom<T>: std::marker::Sized {
    type Error;

    fn try_from(item: T) -> Result<Self, Self::Error>;
}

pub fn to_camel_case(input: &str) -> String {
    let mut result = String::new();
    let mut capitalize = false;
    let mut first = true;
    for ch in input.chars() {
        if ch == '_' {
            capitalize = true;
        } else {
            result.push(if capitalize && !first {
                ch.to_ascii_uppercase()
            } else {
                ch
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
    pub fn with_name(name: &str, span: proc_macro2::Span) -> Self {
        AttributeArguments {
            span: span.unstable(),
            list: vec![(Ident::new("name", span), Lit::Str(LitStr::new(name, span)))],
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "unstable")] {
        pub struct MacroError {
            inner: Diagnostic,
        }
        impl MacroError {
            pub fn emit(self) {
                println!("A");
                self.inner.emit()
            }
        }
        impl std::convert::Into<MacroError> for String {
            fn into(self) -> MacroError {
                MacroError {
                    inner: Span::call_site().error(self)
                }
            }
        }
        impl std::convert::Into<MacroError> for (Span2, &str) {
            fn into(self) -> MacroError {
                MacroError {
                    inner: self.0.unstable().error(self.1)
                }
            }
        }
        impl std::convert::Into<MacroError> for (Span, &str) {
            fn into(self) -> MacroError {
                MacroError {
                    inner: self.0.error(self.1)
                }
            }
        }
    } else {
        pub struct MacroError {
            message: String,
        }
        impl MacroError {
            pub fn emit(self) {
                println!("B");
                panic!("{}", &self.message)
            }
        }
        impl std::convert::Into<MacroError> for String {
            fn into(self) -> MacroError {
                MacroError { message: self }
            }
        }
        impl std::convert::Into<MacroError> for (Span2, &str) {
            fn into(self) -> MacroError {
                MacroError {
                    message: self.1.to_owned(),
                }
            }
        }
        impl std::convert::Into<MacroError> for (Span, &str) {
            fn into(self) -> MacroError {
                MacroError {
                    message: self.1.to_owned(),
                }
            }
        }
    }
}

impl TryFrom<TokenStream> for AttributeArguments {
    type Error = MacroError;

    fn try_from(stream: TokenStream) -> Result<Self, Self::Error> {
        if stream.is_empty() {
            return Ok(AttributeArguments {
                span: Span::call_site(),
                list: Vec::new(),
            });
        }

        parse::<AttributeArguments>(stream).map_err(|e| e.to_string().into())
    }
}

impl TryFrom<proc_macro2::TokenStream> for AttributeArguments {
    type Error = MacroError;

    fn try_from(stream: proc_macro2::TokenStream) -> Result<Self, Self::Error> {
        if stream.is_empty() {
            return Ok(AttributeArguments {
                span: Span::call_site(),
                list: Vec::new(),
            });
        }

        parse2::<AttributeArguments>(stream).map_err(|e| e.to_string().into())
    }
}

impl TryFrom<Attribute> for AttributeArguments {
    type Error = MacroError;

    fn try_from(attr: Attribute) -> Result<Self, Self::Error> {
        let span = attr.span();
        let stream = match TokenBuffer::new2(attr.tts)
            .begin()
            .group(Delimiter::Parenthesis)
        {
            Some((tree, _, _)) => tree.token_stream(),
            None => {
                return Err((span, "failed to parse attribute").into());
            }
        };

        let mut args = AttributeArguments::try_from(stream)
            .map_err(|_| (span, "failed to parse attribute").into())?;
        args.span = span.unstable();
        Ok(args)
    }
}

impl parse::Parse for AttributeArguments {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let exprs = Punctuated::<ArgumentAssignmentExpr, Token![,]>::parse_terminated(input)?;

        Ok(AttributeArguments {
            span: Span::call_site(),
            list: exprs.into_iter().fold(Vec::new(), |mut list, expr| {
                let ArgumentAssignmentExpr(name, value) = expr;
                list.push((name, value));
                list
            }),
        })
    }
}

struct ArgumentAssignmentExpr(Ident, Lit);

impl Parse for ArgumentAssignmentExpr {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let name = Ident::parse(input)?;
        input.parse::<Token![=]>()?;
        let value = Lit::parse(input)?;

        Ok(ArgumentAssignmentExpr(name, value))
    }
}

pub struct QuotableBorrowedStr<'a>(pub &'a str);

impl ToTokens for QuotableBorrowedStr<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let s = self.0;
        quote!(::std::borrow::Cow::Borrowed(#s)).to_tokens(tokens);
    }
}

pub struct QuotableOption<T>(pub Option<T>);

impl<T: ToTokens> ToTokens for QuotableOption<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match &self.0 {
            Some(inner) => quote!(Some(#inner)),
            None => quote!(None),
        }
        .to_tokens(tokens);
    }
}

pub struct QuotableDirection(pub codegen::Direction);

impl ToTokens for QuotableDirection {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
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

impl Parse for PathVec {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let paths = Punctuated::<Path, Token![,]>::parse_terminated(input)?;

        Ok(PathVec(paths.into_iter().collect()))
    }
}

impl IntoIterator for PathVec {
    type Item = Path;
    type IntoIter = std::vec::IntoIter<Path>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl TryFrom<TokenStream> for PathVec {
    type Error = MacroError;

    fn try_from(stream: TokenStream) -> Result<Self, Self::Error> {
        if stream.is_empty() {
            return Ok(Self::default());
        }

        parse::<PathVec>(stream).map_err(|e| e.to_string().into())
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
