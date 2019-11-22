use crate::func::{get_generic_argument_type, FunctionType, OutputBindings};
use azure_functions_shared::codegen::{bindings::TRIGGERS, last_segment_in_path};
use azure_functions_shared::util::to_camel_case;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{FnArg, Ident, ItemFn, Pat, Type};

const INVOKER_PREFIX: &str = "__invoke_";

pub struct Invoker<'a> {
    pub func: &'a ItemFn,
    pub func_type: FunctionType,
}

impl<'a> Invoker<'a> {
    pub fn name(&self) -> String {
        format!("{}{}", INVOKER_PREFIX, self.func.sig.ident)
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

    fn get_args_for_call(&self) -> Vec<TokenStream> {
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
        self.func.sig.inputs.iter().map(|x| match x {
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

    fn get_parameter_mapping(&self) -> TokenStream {
        let (args, types) = self.get_input_args();
        let args_for_match = args.clone();
        let arg_assignments = self.get_input_assignments();
        let arg_names: Vec<_> = args.iter().map(|x| to_camel_case(&x.to_string())).collect();

        let (trigger_arg, trigger_type) = self
            .get_trigger_arg()
            .expect("the function must have a trigger");
        let trigger_name = to_camel_case(&trigger_arg.to_string());

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
                            __metadata.take().expect("expected only one trigger"),
                        )
                    ),
                    #(#arg_names => #args_for_match = Some(#arg_assignments),)*
                    _ => panic!(format!("unexpected parameter binding '{}'", __param.name)),
                };
            }
        )
    }

    fn get_orchestration_invoker(&self, ident: Ident) -> TokenStream {
        let parameter_mapping = self.get_parameter_mapping();
        let args_for_call = self.get_args_for_call();
        let target = &self.func.sig.ident;
        let (trigger, _) = self
            .get_trigger_arg()
            .expect("the function must have a trigger");

        quote!(
            #[allow(dead_code)]
            fn #ident(
                __req: ::azure_functions::rpc::InvocationRequest,
            ) -> ::azure_functions::rpc::InvocationResponse {
                #parameter_mapping

                let __state = #trigger.as_ref().unwrap()._state();

                ::azure_functions::durable::orchestrate(
                    __req.invocation_id,
                    #target(#(#args_for_call,)*),
                    __state,
                )
            }
        )
    }

    fn get_async_entity_invoker(&self, ident: Ident) -> TokenStream {
        unimplemented!()
    }

    fn get_entity_invoker(&self, ident: Ident) -> TokenStream {
        let parameter_mapping = self.get_parameter_mapping();
        let target = &self.func.sig.ident;
        let (trigger, _) = self
            .get_trigger_arg()
            .expect("the function must have a trigger");

        quote!(
            #[allow(dead_code)]
            fn #ident(
                __req: ::azure_functions::rpc::InvocationRequest,
            ) -> ::azure_functions::rpc::InvocationResponse {
                #parameter_mapping

                ::azure_functions::durable::run_entity(
                    __req.invocation_id,
                    #target,
                    #trigger.expect("the function must have a trigger"),
                )
            }
        )
    }

    fn get_async_invoker(&self, ident: Ident) -> TokenStream {
        let async_ident = Ident::new(&format!("{}_async", ident), self.func.sig.ident.span());
        let parameter_mapping = self.get_parameter_mapping();
        let args_for_call = self.get_args_for_call();
        let target = &self.func.sig.ident;
        let output_bindings = OutputBindings { func: self.func };

        quote!(
            #[allow(dead_code)]
            async fn #async_ident(
                __req: ::azure_functions::rpc::InvocationRequest,
            ) -> ::azure_functions::rpc::InvocationResponse {
                #parameter_mapping

                let __ret = #target(#(#args_for_call,)*).await;

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

            #[allow(dead_code)]
            fn #ident(
                __req: ::azure_functions::rpc::InvocationRequest,
            ) -> ::azure_functions::codegen::InvocationFuture {
                std::boxed::Box::pin(#async_ident(__req))
            }
        )
    }

    fn get_invoker(&self, ident: Ident) -> TokenStream {
        let parameter_mapping = self.get_parameter_mapping();
        let args_for_call = self.get_args_for_call();
        let target = &self.func.sig.ident;
        let output_bindings = OutputBindings { func: self.func };

        quote!(
            #[allow(dead_code)]
            fn #ident(
                __req: ::azure_functions::rpc::InvocationRequest,
            ) -> ::azure_functions::rpc::InvocationResponse {
                #parameter_mapping

                let __ret = #target(#(#args_for_call,)*);

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
    }
}

impl ToTokens for Invoker<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = Ident::new(
            &format!("{}{}", INVOKER_PREFIX, self.func.sig.ident.to_string()),
            self.func.sig.ident.span(),
        );

        match self.func_type {
            FunctionType::Orchestration => self.get_orchestration_invoker(ident),
            FunctionType::Entity if self.func.sig.asyncness.is_some() => {
                self.get_async_entity_invoker(ident)
            }
            FunctionType::Entity => self.get_entity_invoker(ident),
            _ if self.func.sig.asyncness.is_some() => self.get_async_invoker(ident),
            _ => self.get_invoker(ident),
        }
        .to_tokens(tokens);
    }
}
