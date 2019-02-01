use crate::util::{last_segment_in_path, to_camel_case};
use quote::quote;
use quote::ToTokens;
use syn::ItemFn;
use syn::{FnArg, GenericArgument, Ident, Pat, PathArguments, ReturnType, Type, TypeReference};

pub struct OutputBindings<'a>(pub &'a ItemFn);

impl<'a> OutputBindings<'a> {
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

    pub fn iter_output_return_bindings(&self) -> Vec<::proc_macro2::TokenStream> {
        match &self.0.decl.output {
            ReturnType::Default => vec![],
            ReturnType::Type(_, ty) => {
                match &**ty {
                    Type::Tuple(tuple) => tuple.elems.iter().enumerate().skip(1).filter_map(|(i, ty)| {
                        if OutputBindings::is_unit_tuple(ty) {
                            return None;
                        }
                        let name = format!("{}{}", crate::func::OUTPUT_BINDING_PREFIX, i);

                        if OutputBindings::is_option_type(ty) {
                            Some(quote!(
                                if let Some(__ret) = __ret.#i {
                                    let mut __output_binding = ::azure_functions::rpc::protocol::ParameterBinding::new();
                                    __output_binding.set_name(#name.to_string());
                                    __output_binding.set_data(__ret.into());
                                    __output_data.push(__output_binding);
                                }
                            ))
                        } else {
                            Some(quote!(
                                let mut __output_binding = ::azure_functions::rpc::protocol::ParameterBinding::new();
                                __output_binding.set_name(#name.to_string());
                                __output_binding.set_data(__ret.#i.into());
                                __output_data.push(__output_binding);
                            ))
                        }
                    }).collect(),
                    _ => vec![],
                }
            }
        }
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
                        tr.mutability?;
                        tr
                    }
                    _ => panic!("expected a type reference"),
                };

                Some((name, arg_type))
            }
            _ => panic!("expected captured arguments"),
        })
    }

    fn is_option_type(t: &Type) -> bool {
        match t {
            Type::Path(tp) => {
                let last = last_segment_in_path(&tp.path);
                if last.ident != "Option" {
                    return false;
                }

                match &last.arguments {
                    PathArguments::AngleBracketed(gen_args) => {
                        if gen_args.args.len() != 1 {
                            return false;
                        }
                        match gen_args.args.iter().nth(0) {
                            Some(GenericArgument::Type(_)) => true,
                            _ => false,
                        }
                    }
                    _ => false,
                }
            }
            Type::Paren(tp) => OutputBindings::is_option_type(&tp.elem),
            _ => false,
        }
    }

    fn is_unit_tuple(t: &Type) -> bool {
        match t {
            Type::Tuple(tuple) => tuple.elems.is_empty(),
            _ => false,
        }
    }
}

impl ToTokens for OutputBindings<'_> {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        let mut output_bindings = self.get_output_bindings();
        output_bindings.append(&mut self.iter_output_return_bindings());

        if !output_bindings.is_empty() {
            quote!(
                {
                    let mut __output_data = __res.mut_output_data();
                    #(#output_bindings;)*
                }
            )
            .to_tokens(tokens);
        }

        match &self.0.decl.output {
            ReturnType::Default => {}
            ReturnType::Type(_, ty) => {
                if let Type::Tuple(tuple) = &**ty {
                    if let Some(first) = tuple.elems.iter().nth(0) {
                        if !OutputBindings::is_unit_tuple(first) {
                            if OutputBindings::is_option_type(first) {
                                quote!(if let Some(__ret) = __ret.0 {
                                    __res.set_return_value(__ret.into());
                                })
                                .to_tokens(tokens);
                            } else {
                                quote!(__res.set_return_value(__ret.0.into());).to_tokens(tokens);
                            }
                        }
                    }
                } else if !OutputBindings::is_unit_tuple(ty) {
                    if OutputBindings::is_option_type(ty) {
                        quote!(if let Some(__ret) = __ret {
                            __res.set_return_value(__ret.into());
                        })
                        .to_tokens(tokens);
                    } else {
                        quote!(__res.set_return_value(__ret.into());).to_tokens(tokens);
                    }
                }
            }
        }
    }
}
