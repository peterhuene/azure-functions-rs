use crate::codegen::TryFrom;
use proc_macro2::{Delimiter, Span, TokenStream};
use syn::{
    buffer::TokenBuffer,
    parse::{Parse, ParseStream},
    parse2,
    punctuated::Punctuated,
    spanned::Spanned,
    Attribute, Ident, Lit, LitStr, Token,
};

pub struct AttributeArguments {
    pub span: Span,
    pub list: Vec<(Ident, Lit)>,
}

impl AttributeArguments {
    pub fn with_name(name: &str, span: Span) -> Self {
        AttributeArguments {
            span,
            list: vec![(Ident::new("name", span), Lit::Str(LitStr::new(name, span)))],
        }
    }
}

impl TryFrom<TokenStream> for AttributeArguments {
    type Error = (Span, String);

    fn try_from(stream: TokenStream) -> Result<Self, Self::Error> {
        if stream.is_empty() {
            return Ok(AttributeArguments {
                span: Span::call_site(),
                list: Vec::new(),
            });
        }

        parse2::<AttributeArguments>(stream).map_err(|e| (Span::call_site(), e.to_string()))
    }
}

impl TryFrom<Attribute> for AttributeArguments {
    type Error = (Span, String);

    fn try_from(attr: Attribute) -> Result<Self, Self::Error> {
        let span = attr.span();
        let stream = match TokenBuffer::new2(attr.tts)
            .begin()
            .group(Delimiter::Parenthesis)
        {
            Some((tree, _, _)) => tree.token_stream(),
            None => {
                return Err((span, "failed to parse attribute".to_string()));
            }
        };

        let mut args = AttributeArguments::try_from(stream)
            .map_err(|_| (span, "failed to parse attribute".to_string()))?;

        args.span = span;

        Ok(args)
    }
}

impl Parse for AttributeArguments {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
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
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let name = Ident::parse(input)?;
        input.parse::<Token![=]>()?;
        let value = Lit::parse(input)?;

        Ok(ArgumentAssignmentExpr(name, value))
    }
}
