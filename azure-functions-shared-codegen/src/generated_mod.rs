use quote::quote;
use std::env;
use std::path::Path;
use syn::{parse, Ident};

pub fn generated_mod_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ident = parse::<Ident>(input).unwrap();

    let mut path = Path::new(&env::var("OUT_DIR").unwrap()).join(ident.to_string());

    path.set_extension("rs");

    let path = path.to_str().unwrap().to_string();

    quote!(
        #[path = #path]
        mod #ident;
    )
    .into()
}
