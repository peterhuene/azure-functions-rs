use crate::func::get_generic_argument_type;
use azure_functions_shared::{codegen::last_segment_in_path, util::to_camel_case};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{FnArg, Ident, Index, ItemFn, Pat, ReturnType, Type};

pub struct OutputBindings<'a>(pub &'a ItemFn);

impl<'a> OutputBindings<'a> {
    fn get_output_argument_bindings(&self) -> Vec<TokenStream> {
        self.iter_mut_args()
            .map(|(name, _)| {
                let name_str = to_camel_case(&name.to_string());
                quote!(
                    __output_data.push(::azure_functions::rpc::ParameterBinding{
                        name: #name_str.to_string(),
                        data: Some(#name.unwrap().into()),
                    });
                )
            })
            .collect()
    }

    fn get_output_return_binding(ty: &Type, index: Index) -> Option<TokenStream> {
        if OutputBindings::is_unit_tuple(ty) {
            return None;
        }

        let name = format!("{}{}", crate::func::OUTPUT_BINDING_PREFIX, index.index);

        match OutputBindings::get_generic_argument_type(ty, "Option") {
            Some(inner) => {
                let conversion = OutputBindings::get_binding_conversion(inner, None);
                Some(quote!(
                    if let Some(__ret) = __ret.#index {
                        __res.output_data.push(::azure_functions::rpc::ParameterBinding{
                            name: #name.to_string(),
                            data: Some(#conversion)
                        });
                    }
                ))
            }
            None => {
                let conversion = OutputBindings::get_binding_conversion(ty, Some(index));
                Some(quote!(
                    __res.output_data.push(::azure_functions::rpc::ParameterBinding{
                        name: #name.to_string(),
                        data: Some(#conversion)
                    });
                ))
            }
        }
    }

    fn get_binding_conversion(ty: &Type, index: Option<Index>) -> TokenStream {
        match OutputBindings::get_generic_argument_type(ty, "Vec") {
            Some(_) => match index {
                Some(index) => quote!(::azure_functions::rpc::TypedData::from_vec(__ret.#index)),
                None => quote!(::azure_functions::rpc::TypedData::from_vec(__ret)),
            },
            None => match index {
                Some(index) => quote!(__ret.#index.into()),
                None => quote!(__ret.into()),
            },
        }
    }

    fn iter_output_return_bindings(&self) -> Vec<TokenStream> {
        match &self.0.decl.output {
            ReturnType::Default => vec![],
            ReturnType::Type(_, ty) => match &**ty {
                Type::Tuple(tuple) => tuple
                    .elems
                    .iter()
                    .enumerate()
                    .skip(1)
                    .filter_map(|(i, ty)| OutputBindings::get_output_return_binding(ty, i.into()))
                    .collect(),
                _ => vec![],
            },
        }
    }

    fn iter_mut_args(&self) -> impl Iterator<Item = (&'a Ident, &'a Type)> {
        self.0.decl.inputs.iter().filter_map(|x| match x {
            FnArg::Captured(arg) => {
                if let Type::Reference(tr) = &arg.ty {
                    tr.mutability?;

                    let name = match &arg.pat {
                        Pat::Ident(name) => &name.ident,
                        _ => panic!("expected ident argument pattern"),
                    };

                    return Some((name, &arg.ty));
                }
                None
            }
            _ => panic!("expected captured arguments"),
        })
    }

    fn get_generic_argument_type(ty: &'a Type, generic_type_name: &str) -> Option<&'a Type> {
        match ty {
            Type::Path(tp) => {
                get_generic_argument_type(last_segment_in_path(&tp.path), generic_type_name)
            }
            Type::Paren(tp) => {
                OutputBindings::get_generic_argument_type(&tp.elem, generic_type_name)
            }
            _ => None,
        }
    }

    fn is_unit_tuple(t: &Type) -> bool {
        match t {
            Type::Tuple(tuple) => tuple.elems.is_empty(),
            _ => false,
        }
    }

    fn get_return_binding(ty: &Type, in_tuple: bool) -> Option<TokenStream> {
        if OutputBindings::is_unit_tuple(ty) {
            return None;
        }

        if in_tuple {
            match OutputBindings::get_generic_argument_type(ty, "Option") {
                Some(inner) => {
                    let conversion = OutputBindings::get_binding_conversion(inner, None);
                    Some(quote!(
                        if let Some(__ret) = __ret.0 {
                            __res.return_value = Some(#conversion);
                        }
                    ))
                }
                None => {
                    let conversion = OutputBindings::get_binding_conversion(ty, Some(0.into()));
                    Some(quote!(__res.return_value = Some(#conversion);))
                }
            }
        } else {
            if let Type::Tuple(tuple) = &*ty {
                if let Some(first) = tuple.elems.iter().nth(0) {
                    return OutputBindings::get_return_binding(first, true);
                }
                return None;
            }

            match OutputBindings::get_generic_argument_type(ty, "Option") {
                Some(inner) => {
                    let conversion = OutputBindings::get_binding_conversion(inner, None);
                    Some(quote!(
                        if let Some(__ret) = __ret {
                            __res.return_value = Some(#conversion);
                        }
                    ))
                }
                None => {
                    let conversion = OutputBindings::get_binding_conversion(ty, None);
                    Some(quote!(__res.return_value = Some(#conversion);))
                }
            }
        }
    }
}

impl ToTokens for OutputBindings<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for binding in self.get_output_argument_bindings() {
            binding.to_tokens(tokens);
        }

        for binding in self.iter_output_return_bindings() {
            binding.to_tokens(tokens);
        }

        match &self.0.decl.output {
            ReturnType::Default => {}
            ReturnType::Type(_, ty) => {
                if let Some(binding) = OutputBindings::get_return_binding(ty, false) {
                    binding.to_tokens(tokens);
                }
            }
        }
    }
}
