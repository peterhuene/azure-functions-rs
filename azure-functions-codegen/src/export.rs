use azure_functions_shared::codegen::macro_panic;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse,
    parse::{Parse, ParseStream},
    parse_str,
    punctuated::Punctuated,
    spanned::Spanned,
    Path, Token,
};

#[derive(Default)]
struct PathVec(Vec<Path>);

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

impl From<TokenStream> for PathVec {
    fn from(stream: TokenStream) -> Self {
        if stream.is_empty() {
            return Self::default();
        }

        parse::<PathVec>(stream)
            .map_err(|e| macro_panic(Span::call_site(), e.to_string()))
            .unwrap()
    }
}

pub fn export_impl(input: TokenStream) -> TokenStream {
    let mut funcs = Vec::new();
    for path in PathVec::from(input).into_iter() {
        if path.leading_colon.is_some() || path.segments.len() > 1 {
            macro_panic(
                path.span(),
                "fully qualified names are not supported for the `export` macro",
            );
        }

        let segment = path.segments.first().unwrap();
        let identifier = segment.value().ident.to_string();

        funcs.push(
            parse_str::<Path>(&format!(
                "{}::{}_FUNCTION",
                identifier,
                identifier.to_uppercase()
            ))
            .unwrap(),
        );
    }

    quote!(
        pub const EXPORTS: &[&::azure_functions::codegen::Function] = &[#(&#funcs),*];
    )
    .into()
}
