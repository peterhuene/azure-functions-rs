use crate::util::MacroError;
use crate::util::PathVec;
use azure_functions_shared::codegen::TryFrom;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_str, spanned::Spanned, Path};

pub fn attr_impl(input: TokenStream) -> TokenStream {
    let (mods, funcs) = match PathVec::try_from(input.clone()) {
        Ok(paths) => {
            let mut mods: Vec<_> = Vec::new();
            let mut funcs = Vec::new();
            for path in paths.into_iter() {
                if path.leading_colon.is_some() || path.segments.len() > 1 {
                    let error: MacroError = (
                        path.span(),
                        "fully qualified names are not supported for the `export` macro"
                            .to_string(),
                    )
                        .into();

                    error.emit();
                    return input;
                }

                let segment = path.segments.first().unwrap();
                let identifier = segment.value().ident.to_string();

                funcs.push(
                    parse_str::<Path>(&format!(
                        "{}::__{}_FUNCTION",
                        identifier,
                        identifier.to_uppercase()
                    ))
                    .unwrap(),
                );
                mods.push(path);
            }
            (mods, funcs)
        }
        Err(e) => {
            e.emit();
            return input;
        }
    };

    let expanded = quote! {
        #(mod #mods;)*
        pub const FUNCTIONS: &[&::azure_functions::codegen::Function] = &[#(&#funcs),*];
    };

    expanded.into()
}
