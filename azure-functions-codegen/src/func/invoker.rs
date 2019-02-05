use crate::func::TRIGGERS;
use crate::func::{OutputBindings, CONTEXT_TYPE_NAME};
use crate::util::{last_segment_in_path, to_camel_case};
use quote::{quote, ToTokens};
use syn::{FnArg, Ident, ItemFn, Pat, Type, TypeReference};

const INVOKER_PREFIX: &str = "__invoke_";

pub struct Invoker<'a>(pub &'a ItemFn);

impl<'a> Invoker<'a> {
    pub fn name(&self) -> String {
        format!("{}{}", INVOKER_PREFIX, self.0.ident)
    }

    fn get_args(&self) -> (Vec<&'a Ident>, Vec<&'a Type>) {
        self.iter_args()
            .filter_map(|(name, arg_type)| {
                if Invoker::is_context_type(&arg_type.elem) {
                    return None;
                }

                Some((name, &*arg_type.elem))
            })
            .unzip()
    }

    fn get_trigger_arg(&self) -> Option<(&'a Ident, &'a Type)> {
        self.iter_args()
            .find(|(_, arg_type)| Invoker::is_trigger_type(&arg_type.elem))
            .map(|(name, arg_type)| (name, &*arg_type.elem))
    }

    fn get_args_for_call(&self) -> Vec<::proc_macro2::TokenStream> {
        self.iter_args()
            .map(|(name, arg_type)| {
                if Invoker::is_context_type(&arg_type.elem) {
                    return quote!(__ctx);
                }

                let name_str = name.to_string();
                match arg_type.mutability {
                    Some(_) => quote!(#name.as_mut().expect(concat!("parameter binding '", #name_str, "' was not provided"))),
                    None => quote!(#name.as_ref().expect(concat!("parameter binding '", #name_str, "' was not provided")))
                }
            })
            .collect()
    }

    fn iter_args(&self) -> impl Iterator<Item = (&'a Ident, &'a TypeReference)> {
        self.0.decl.inputs.iter().map(|x| match x {
            FnArg::Captured(arg) => (
                match &arg.pat {
                    Pat::Ident(name) => &name.ident,
                    _ => panic!("expected ident argument pattern"),
                },
                match &arg.ty {
                    Type::Reference(tr) => tr,
                    _ => panic!("expected a type reference"),
                },
            ),
            _ => panic!("expected captured arguments"),
        })
    }

    fn is_context_type(ty: &Type) -> bool {
        match ty {
            Type::Path(tp) => last_segment_in_path(&tp.path).ident == CONTEXT_TYPE_NAME,
            Type::Paren(tp) => Invoker::is_context_type(&tp.elem),
            _ => false,
        }
    }

    fn is_trigger_type(ty: &Type) -> bool {
        match ty {
            Type::Path(tp) => {
                TRIGGERS.contains_key(last_segment_in_path(&tp.path).ident.to_string().as_str())
            }
            Type::Paren(tp) => Invoker::is_trigger_type(&tp.elem),
            _ => false,
        }
    }
}

impl ToTokens for Invoker<'_> {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        let invoker = Ident::new(
            &format!("{}{}", INVOKER_PREFIX, self.0.ident.to_string()),
            self.0.ident.span(),
        );
        let target = &self.0.ident;

        let (args, arg_types) = self.get_args();
        let args_for_match = args.clone();
        let (trigger_arg, _) = self
            .get_trigger_arg()
            .expect("the function must have a trigger");
        let binding_names: Vec<_> = args.iter().map(|x| to_camel_case(&x.to_string())).collect();

        let args_for_call = self.get_args_for_call();

        let output_bindings = OutputBindings(self.0);

        quote!(#[allow(dead_code)]
        fn #invoker(
            __name: &str,
            __req: &mut ::azure_functions::rpc::protocol::InvocationRequest,
        ) -> ::azure_functions::rpc::protocol::InvocationResponse {

            #(let mut #args: Option<#arg_types> = None;)*

            for __param in __req.input_data.iter_mut() {
                match __param.name.as_str() {
                   #(#binding_names => #args_for_match = Some(__param.take_data().into()),)*
                    _ => panic!(format!("unexpected parameter binding '{}'", __param.name)),
                };
            }

            use ::azure_functions::bindings::Trigger;
            match #trigger_arg.as_mut() {
                Some(t) => t.read_metadata(&mut __req.trigger_metadata),
                None => {}
            };

            let __ctx = &::azure_functions::Context::new(&__req.invocation_id, &__req.function_id, __name);
            let __ret = #target(#(#args_for_call,)*);

            let mut __res = ::azure_functions::rpc::protocol::InvocationResponse::new();
            __res.set_invocation_id(__req.invocation_id.clone());
            #output_bindings
            __res.mut_result().status =
                ::azure_functions::rpc::protocol::StatusResult_Status::Success;

            __res
        }).to_tokens(tokens);
    }
}
