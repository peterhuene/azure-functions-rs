use crate::func::{get_generic_argument_type, OutputBindings};
use azure_functions_shared::codegen::{bindings::TRIGGERS, last_segment_in_path};
use azure_functions_shared::util::to_camel_case;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{FnArg, Ident, ItemFn, Pat, Type};

const INVOKER_PREFIX: &str = "__invoke_";

pub struct Invoker<'a>(pub &'a ItemFn);

impl<'a> Invoker<'a> {
    pub fn name(&self) -> String {
        format!("{}{}", INVOKER_PREFIX, self.0.sig.ident)
    }

    fn deref_arg_type(ty: &Type) -> &Type {
        match ty {
            Type::Reference(tr) => &*tr.elem,
            _ => ty,
        }
    }

    fn is_trigger_type(ty: &Type) -> bool {
        match Invoker::deref_arg_type(ty) {
            Type::Path(tp) => {
                TRIGGERS.contains_key(last_segment_in_path(&tp.path).ident.to_string().as_str())
            }
            Type::Paren(tp) => Invoker::is_trigger_type(&tp.elem),
            _ => false,
        }
    }
}

struct CommonInvokerTokens<'a>(pub &'a ItemFn);

impl<'a> CommonInvokerTokens<'a> {
    fn get_input_args(&self) -> (Vec<&'a Ident>, Vec<&'a Type>) {
        self.iter_args()
            .filter_map(|(name, arg_type)| {
                if Invoker::is_trigger_type(arg_type) {
                    return None;
                }

                Some((name, Invoker::deref_arg_type(arg_type)))
            })
            .unzip()
    }

    fn get_input_assignments(&self) -> Vec<TokenStream> {
        self.iter_args()
            .filter_map(|(_, arg_type)| {
                if Invoker::is_trigger_type(arg_type) {
                    return None;
                }

                if let Type::Path(tp) = Invoker::deref_arg_type(arg_type) {
                    if get_generic_argument_type(last_segment_in_path(&tp.path), "Vec").is_some() {
                        return Some(quote!(__param
                            .data
                            .expect("expected parameter binding data")
                            .into_vec()));
                    }
                }

                Some(quote!(__param
                    .data
                    .expect("expected parameter binding data")
                    .into()))
            })
            .collect()
    }

    fn get_trigger_arg(&self) -> Option<(&'a Ident, &'a Type)> {
        self.iter_args()
            .find(|(_, arg_type)| Invoker::is_trigger_type(arg_type))
            .map(|(name, arg_type)| (name, Invoker::deref_arg_type(arg_type)))
    }

    fn get_args_for_call(&self) -> Vec<::proc_macro2::TokenStream> {
        self.iter_args()
            .map(|(name, arg_type)| {
                let name_str = name.to_string();

                if let Type::Reference(tr) = arg_type {
                    return match tr.mutability {
                        Some(_) => quote!(#name.as_mut().expect(concat!("parameter binding '", #name_str, "' was not provided"))),
                        None => quote!(#name.as_ref().expect(concat!("parameter binding '", #name_str, "' was not provided")))
                    };
                }

                quote!(#name.expect(concat!("parameter binding '", #name_str, "' was not provided")))
            })
            .collect()
    }

    fn iter_args(&self) -> impl Iterator<Item = (&'a Ident, &'a Type)> {
        self.0.sig.inputs.iter().map(|x| match x {
            FnArg::Typed(arg) => (
                match &*arg.pat {
                    Pat::Ident(name) => &name.ident,
                    _ => panic!("expected ident argument pattern"),
                },
                &*arg.ty,
            ),
            _ => panic!("expected captured arguments"),
        })
    }
}

impl ToTokens for CommonInvokerTokens<'_> {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        let target = &self.0.sig.ident;

        let (args, types) = self.get_input_args();
        let args_for_match = args.clone();
        let arg_assignments = self.get_input_assignments();
        let arg_names: Vec<_> = args.iter().map(|x| to_camel_case(&x.to_string())).collect();

        let (trigger_arg, trigger_type) = self
            .get_trigger_arg()
            .expect("the function must have a trigger");
        let trigger_name = to_camel_case(&trigger_arg.to_string());

        let args_for_call = self.get_args_for_call();

        quote!(
            use azure_functions::{IntoVec, FromVec};

            let mut #trigger_arg: Option<#trigger_type> = None;
            #(let mut #args: Option<#types> = None;)*

            let mut __metadata = Some(__req.trigger_metadata);

            for __param in __req.input_data.into_iter() {
                match __param.name.as_str() {
                    #trigger_name => #trigger_arg = Some(
                        #trigger_type::new(
                            __param.data.expect("expected parameter binding data"),
                            __metadata.take().expect("expected only one trigger")
                        )
                    ),
                    #(#arg_names => #args_for_match = Some(#arg_assignments),)*
                    _ => panic!(format!("unexpected parameter binding '{}'", __param.name)),
                };
            }

            let __ret = #target(#(#args_for_call,)*);
        )
        .to_tokens(tokens);
    }
}

impl ToTokens for Invoker<'_> {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        let ident = Ident::new(
            &format!("{}{}", INVOKER_PREFIX, self.0.sig.ident.to_string()),
            self.0.sig.ident.span(),
        );

        let common_tokens = CommonInvokerTokens(&self.0);

        let output_bindings = OutputBindings(self.0);

        if self.0.sig.asyncness.is_some() {
            quote!(
                #[allow(dead_code)]
                fn #ident(
                    __req: ::azure_functions::rpc::InvocationRequest,
                ) -> ::azure_functions::codegen::InvocationFuture {
                    #common_tokens

                    use futures::future::FutureExt;

                    let __id = __req.invocation_id;

                    Box::pin(
                        __ret.then(move |__ret| {
                            let mut __res = ::azure_functions::rpc::InvocationResponse {
                                invocation_id: __id,
                                result: Some(::azure_functions::rpc::StatusResult {
                                    status: ::azure_functions::rpc::status_result::Status::Success as i32,
                                    ..Default::default()
                                }),
                                ..Default::default()
                            };

                            #output_bindings

                            ::futures::future::ready(__res)
                        })
                    )
                }
            ).to_tokens(tokens);
        } else {
            quote!(
                #[allow(dead_code)]
                fn #ident(
                    __req: ::azure_functions::rpc::InvocationRequest,
                ) -> ::azure_functions::rpc::InvocationResponse {
                    #common_tokens

                    let mut __res = ::azure_functions::rpc::InvocationResponse {
                        invocation_id: __req.invocation_id,
                        result: Some(::azure_functions::rpc::StatusResult {
                            status: ::azure_functions::rpc::status_result::Status::Success as i32,
                            ..Default::default()
                        }),
                        ..Default::default()
                    };

                    #output_bindings

                    __res
                }
            )
            .to_tokens(tokens);
        }
    }
}
