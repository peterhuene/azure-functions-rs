use azure_functions_shared::codegen::{last_segment_in_path, macro_panic};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Ident, Path, Token,
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
    for mut path in PathVec::from(input).into_iter() {
        let last = last_segment_in_path(&path);
        let identifier = Ident::new(
            &format!("{}_FUNCTION", last.ident.to_string().to_uppercase()),
            last.span(),
        );

        path.segments.pop();

        funcs.push(quote!(#path#identifier));
    }

    quote!(
        pub const EXPORTS: &[&::azure_functions::codegen::Function] = &[#(&#funcs),*];
    )
    .into()
}
