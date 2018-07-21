use func::TRIGGERS;
use func::{ReturnValueSetter, CONTEXT_TYPE_NAME};
use quote::ToTokens;
use syn::{FnArg, Ident, ItemFn, Pat, Type};
use util::{last_ident_in_path, to_camel_case};

const INVOKER_PREFIX: &'static str = "__invoke_";

pub struct Invoker<'a>(pub &'a ItemFn);

impl<'a> Invoker<'a> {
    pub fn name(&self) -> String {
        format!("{}{}", INVOKER_PREFIX, self.0.ident)
    }
}

impl<'a> Invoker<'a> {
    fn get_args(&self) -> (Vec<&'a Ident>, Vec<&'a Type>) {
        self.iter_args()
            .filter_map(|(name, arg_type)| {
                if let Type::Path(tp) = arg_type {
                    if last_ident_in_path(&tp.path) == CONTEXT_TYPE_NAME {
                        return None;
                    }
                }

                Some((name, arg_type))
            })
            .unzip()
    }

    fn get_trigger_arg(&self) -> Option<(&'a Ident, &'a Type)> {
        self.iter_args().find(|(_, arg_type)| {
            if let Type::Path(tp) = arg_type {
                return TRIGGERS.contains_key(last_ident_in_path(&tp.path).as_str());
            }
            false
        })
    }

    fn get_args_for_call(&self) -> Vec<::proc_macro2::TokenStream> {
        self.iter_args()
            .map(|(name, arg_type)| {
                if let Type::Path(tp) = arg_type {
                    if last_ident_in_path(&tp.path) == CONTEXT_TYPE_NAME {
                        return quote!(__ctx);
                    }
                }

                let name_str = name.to_string();
                quote!(&#name.expect(concat!("parameter binding '", #name_str, "' was not provided")))
            })
            .collect()
    }

    fn iter_args(&self) -> impl Iterator<Item = (&'a Ident, &'a Type)> {
        self.0.decl.inputs.iter().map(|x| match x {
            FnArg::Captured(arg) => (
                match &arg.pat {
                    Pat::Ident(name) => &name.ident,
                    _ => panic!("expected ident argument pattern"),
                },
                match &arg.ty {
                    Type::Reference(tr) => &*tr.elem,
                    _ => panic!("expected a type reference"),
                },
            ),
            _ => panic!("expected captured arguments"),
        })
    }
}

impl<'a> ToTokens for Invoker<'a> {
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

        let return_value = ReturnValueSetter(&self.0.decl.output);

        quote!(#[allow(dead_code)]
        fn #invoker(
            __req: &::azure_functions::rpc::protocol::InvocationRequest,
            __ctx: &::azure_functions::Context
        ) -> ::azure_functions::rpc::protocol::InvocationResponse {
            #(let mut #args: Option<#arg_types> = None;)*

            for __param in __req.input_data.iter() {
                match __param.name.as_str() {
                    #(#binding_names => #args_for_match = Some(__param.data.get_ref().into()),)*
                    _ => panic!(format!("unexpected parameter binding '{}'", __param.name)),
                };
            }

            use ::azure_functions::bindings::Trigger;
            match #trigger_arg.as_mut() {
                Some(t) => t.read_metadata(&__req.trigger_metadata),
                None => {}
            };

            let __ret = #target(#(#args_for_call,)*);

            let mut __res = ::azure_functions::rpc::protocol::InvocationResponse::new();
            __res.set_invocation_id(__req.invocation_id.clone());
            #return_value
            __res.mut_result().status =
                ::azure_functions::rpc::protocol::StatusResult_Status::Success;

            __res
            }
        ).to_tokens(tokens);
    }
}
