use quote::ToTokens;
use syn::ItemFn;
use syn::{FnArg, Ident, Pat, ReturnType, Type, TypeReference};
use util::to_camel_case;

pub struct OutputBindings<'a>(pub &'a ItemFn);

impl OutputBindings<'a> {
    fn get_output_bindings(&self) -> Vec<::proc_macro2::TokenStream> {
        self.iter_mut_args()
            .map(|(name, _)| {
                let name_str = to_camel_case(&name.to_string());
                quote!(
                    let mut __output_binding = ::azure_functions::rpc::protocol::ParameterBinding::new();
                    __output_binding.set_name(#name_str.to_string());
                    __output_binding.set_data(#name.unwrap().into());
                    __output_data.push(__output_binding);
                )
            })
            .collect()
    }

    fn iter_mut_args(&self) -> impl Iterator<Item = (&'a Ident, &'a TypeReference)> {
        self.0.decl.inputs.iter().filter_map(|x| match x {
            FnArg::Captured(arg) => {
                let name = match &arg.pat {
                    Pat::Ident(name) => &name.ident,
                    _ => panic!("expected ident argument pattern"),
                };

                let arg_type = match &arg.ty {
                    Type::Reference(tr) => {
                        if tr.mutability.is_none() {
                            return None;
                        }
                        tr
                    }
                    _ => panic!("expected a type reference"),
                };

                Some((name, arg_type))
            }
            _ => panic!("expected captured arguments"),
        })
    }
}

impl ToTokens for OutputBindings<'_> {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        // Set the output bindings
        let output_bindings = self.get_output_bindings();

        quote!(
            {
                let mut __output_data = __res.mut_output_data();
                #(#output_bindings;)*
            }
        ).to_tokens(tokens);

        // TODO: support tuple return types
        // First value (or last maybe?) will be $return, the other values will be output bindings
        match &self.0.decl.output {
            ReturnType::Default => {}
            ReturnType::Type(_, _) => {
                quote!(__res.set_return_value(__ret.into());).to_tokens(tokens);
            }
        };
    }
}
