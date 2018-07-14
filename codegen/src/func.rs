use bindings::{Binding, INPUT_BINDINGS, INPUT_OUTPUT_BINDINGS, OUTPUT_BINDINGS, TRIGGERS};
use proc_macro::{Diagnostic, TokenStream};
use proc_macro2::Span;
use quote::ToTokens;
use std::collections::HashMap;
use std::convert::TryFrom;
use syn::spanned::Spanned;
use syn::{parse, Attribute, FnArg, Ident, ItemFn, Lit, Pat, ReturnType, Type, Visibility};
use util::{AttributeArguments, ToString};

const RETURN_BINDING_NAME: &'static str = "$return";
const CONTEXT_TYPE_NAME: &'static str = "Context";

#[derive(Default, Debug)]
struct Function {
    pub name: String,
    pub disabled: bool,
    pub bindings: Vec<Binding>,
    pub callback: Option<Ident>,
}

impl TryFrom<TokenStream> for Function {
    type Error = Diagnostic;

    fn try_from(stream: TokenStream) -> Result<Self, Self::Error> {
        let mut name = None;
        let mut disabled = None;

        for (key, value) in AttributeArguments::try_from(stream)?.0.into_iter() {
            let key_str = key.to_string();

            match key_str.as_str() {
                "name" => match &value {
                    Lit::Str(s) => {
                        name = s
                            .parse::<Ident>()
                            .map(|x| Some(x.to_string()))
                            .map_err(|_| {
                                value.span().unstable().error(
                                "a legal function identifier is required for the 'name' argument",
                            )
                            })?;
                    }
                    _ => {
                        return Err(value
                            .span()
                            .unstable()
                            .error("expected a literal string value for the 'name' argument"));
                    }
                },
                "disabled" => match value {
                    Lit::Bool(b) => disabled = Some(b.value),
                    _ => {
                        return Err(value
                            .span()
                            .unstable()
                            .error("expected a literal boolean value for the 'disabled' argument"));
                    }
                },
                _ => {
                    return Err(key
                        .span()
                        .unstable()
                        .error(format!("unsupported argument '{}'", key_str)));
                }
            };
        }

        Ok(Function {
            name: name.unwrap_or(String::new()),
            disabled: disabled.unwrap_or(false),
            bindings: Vec::new(),
            callback: None,
        })
    }
}

impl ToTokens for Function {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        let name = &self.name;
        let disabled = &self.disabled;
        let bindings = self.bindings.iter().filter(|x| !x.is_context());
        let callback = self
            .callback
            .as_ref()
            .expect("expected a callback for the function");
        quote!(
        ::azure_functions::codegen::Function {
            name: #name,
            disabled: #disabled,
            bindings: &[#(&#bindings),*],
            callback: #callback,
        }
        ).to_tokens(tokens)
    }
}

struct TargetInvoker<'a>(&'a ItemFn);

impl<'a> TargetInvoker<'a> {
    fn get_args(&self) -> (Vec<&'a Ident>, Vec<&'a Type>) {
        self.iter_args()
            .filter_map(|(name, arg_type)| {
                if let Type::Path(tp) = arg_type {
                    if tp.path.to_string() == CONTEXT_TYPE_NAME {
                        return None;
                    }
                }

                Some((name, arg_type))
            })
            .unzip()
    }

    fn get_args_for_call(&self) -> Vec<::proc_macro2::TokenStream> {
        self.iter_args()
            .map(|(name, arg_type)| {
                if let Type::Path(tp) = arg_type {
                    if tp.path.to_string() == CONTEXT_TYPE_NAME {
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

impl<'a> ToTokens for TargetInvoker<'a> {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        let invoker = Ident::new(
            &format!("__invoke_{}", self.0.ident.to_string()),
            self.0.ident.span(),
        );
        let target = &self.0.ident;

        let (args, arg_types) = self.get_args();
        let args_for_match = args.clone();
        let arg_names: Vec<_> = args.iter().map(|x| x.to_string()).collect();

        let args_for_call = self.get_args_for_call();

        let output = OutputSetter(&self.0.decl.output);

        quote!(#[allow(dead_code)]
        fn #invoker(
            __req: &::azure_functions::rpc::protocol::InvocationRequest,
            __ctx: &::azure_functions::Context
        ) -> ::azure_functions::rpc::protocol::InvocationResponse {
            #(let mut #args: Option<#arg_types> = None;)*

            for __param in __req.input_data.iter() {
                match __param.name.as_str() {
                    #(#arg_names => #args_for_match = Some(__param.data.get_ref().into()),)*
                    _ => panic!(format!("unexpected parameter binding '{}'", __param.name)),
                };
            }

            let __ret = #target(#(#args_for_call,)*);

            let mut __res = ::azure_functions::rpc::protocol::InvocationResponse::new();
            __res.set_invocation_id(__req.invocation_id.clone());
            #output
            __res.mut_result().status =
                ::azure_functions::rpc::protocol::StatusResult_Status::Success;

            __res
            }
        ).to_tokens(tokens);
    }
}

struct OutputSetter<'a>(&'a ReturnType);

impl<'a> ToTokens for OutputSetter<'a> {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        match self.0 {
            ReturnType::Default => {}
            ReturnType::Type(_, _) => {
                quote!(__res.set_return_value(__ret.into());).to_tokens(tokens);
            }
        };
    }
}

fn validate_function(func: &ItemFn) -> Result<(), Diagnostic> {
    match func.vis {
        Visibility::Public(_) => {}
        _ => {
            return Err(func
                .decl
                .fn_token
                .span()
                .unstable()
                .error("the 'func' attribute can only be used on public functions"))
        }
    };

    if func.abi.is_some() {
        return Err(func
            .abi
            .as_ref()
            .unwrap()
            .extern_token
            .span()
            .unstable()
            .error("the 'func' attribute cannot be used on extern \"C\" functions"));
    }

    if func.constness.is_some() {
        return Err(func
            .constness
            .as_ref()
            .unwrap()
            .0
            .unstable()
            .error("the 'func' attribute cannot be used on const functions"));
    }

    if func.unsafety.is_some() {
        return Err(func
            .unsafety
            .as_ref()
            .unwrap()
            .0
            .unstable()
            .error("the 'func' attribute cannot be used on unsafe functions"));
    }

    if !func.decl.generics.params.is_empty() {
        return Err(func
            .decl
            .generics
            .params
            .span()
            .unstable()
            .error("the 'func' attribute cannot be used on generic functions"));
    }

    if func.decl.variadic.is_some() {
        return Err(func
            .decl
            .variadic
            .span()
            .unstable()
            .error("the 'func' attribute cannot be used on variadic functions"));
    }

    Ok(())
}

fn bind_argument(
    arg: &FnArg,
    has_trigger: bool,
    binding_args: &mut HashMap<String, AttributeArguments>,
) -> Result<Binding, Diagnostic> {
    match arg {
        FnArg::Captured(arg) => match &arg.ty {
            Type::Reference(r) => match &*r.elem {
                Type::Path(tp) => {
                    let type_name = tp.path.to_string();

                    if type_name == CONTEXT_TYPE_NAME {
                        return Ok(Binding::Context);
                    }

                    let factory = match TRIGGERS.get(type_name.as_str()) {
                        Some(factory) => match r.mutability {
                            Some(m) => Err(m
                                .span()
                                .unstable()
                                .error("trigger arguments cannot be passed by mutable reference")),
                            None => {
                                if has_trigger {
                                    Err(tp
                                        .span()
                                        .unstable()
                                        .error("Azure Functions can only have one trigger binding"))
                                } else {
                                    Ok(factory)
                                }
                            }
                        },
                        None => match INPUT_BINDINGS.get(type_name.as_str()) {
                            Some(factory) => match r.mutability {
                                Some(m) => Err(m
                                    .span()
                                    .unstable()
                                    .error("input arguments cannot be passed by mutable reference")),
                                None => Ok(factory),
                            },
                            None => match INPUT_OUTPUT_BINDINGS.get(type_name.as_str()) {
                                Some(factory) => match r.mutability {
                                    Some(_) => Ok(factory),
                                    None => Err(r
                                        .span()
                                        .unstable()
                                        .error("input-output arguments must be passed by mutable reference")),
                                },
                                None => Err(tp
                                    .span()
                                    .unstable()
                                    .error("expected an Azure Function trigger or input binding type"))
                            },
                        },
                    }?;
                    match &arg.pat {
                        Pat::Ident(name) => {
                            let name_str = name.ident.to_string();
                            match binding_args.remove(&name_str) {
                                Some(args) => (*factory)(&args),
                                None => (*factory)(&AttributeArguments::with_name(
                                    &name_str,
                                    name.span(),
                                )),
                            }
                        }
                        _ => Err(arg
                            .pat
                            .span()
                            .unstable()
                            .error("arguments must have a named binding")),
                    }
                }
                _ => Err(arg
                    .ty
                    .span()
                    .unstable()
                    .error("expected an Azure Function trigger or input binding type")),
            },
            _ => Err(arg.ty.span().unstable().error(
                "expected an Azure Function trigger or input binding type passed by reference",
            )),
        },
        FnArg::SelfRef(_) | FnArg::SelfValue(_) => Err(arg
            .span()
            .unstable()
            .error("Azure Functions cannot have self arguments")),
        FnArg::Inferred(_) => Err(arg
            .span()
            .unstable()
            .error("Azure Functions cannot have inferred arguments")),
        FnArg::Ignored(_) => Err(arg
            .span()
            .unstable()
            .error("Azure Functions cannot have ignored arguments")),
    }
}

fn bind_return_type(
    ret: &ReturnType,
    binding_args: &mut HashMap<String, AttributeArguments>,
) -> Result<Option<Binding>, Diagnostic> {
    match ret {
        ReturnType::Default => Ok(None),
        ReturnType::Type(_, ty) => match &**ty {
            Type::Path(tp) => match OUTPUT_BINDINGS.get(tp.path.to_string().as_str()) {
                Some(factory) => match binding_args.remove(RETURN_BINDING_NAME) {
                    Some(args) => (*factory)(&args),
                    None => (*factory)(&AttributeArguments::with_name(
                        RETURN_BINDING_NAME,
                        ty.span(),
                    )),
                }.map(|x| Some(x)),
                None => Err(tp
                    .span()
                    .unstable()
                    .error("expected an Azure Function output binding type")),
            },
            _ => Err(ty
                .span()
                .unstable()
                .error("expected an Azure Function output binding type")),
        },
    }
}

fn drain_binding_attributes(
    attrs: &mut Vec<Attribute>,
) -> Result<HashMap<String, AttributeArguments>, Diagnostic> {
    let mut map = HashMap::new();
    for attr in attrs.drain_filter(|a| a.path.to_string() == "binding") {
        let attr_span = attr.span();
        let args = AttributeArguments::try_from(attr)?;

        let (name, name_span) = match args.0.iter().find(|(k, _)| k.to_string() == "name") {
            Some((_, v)) => match v {
                Lit::Str(s) => (s.value(), s.span()),
                _ => {
                    return Err(v
                        .span()
                        .unstable()
                        .error("expected a literal string value for the 'name' argument"));
                }
            },
            None => {
                return Err(attr_span
                    .unstable()
                    .error("binding attributes must have a 'name' argument"));
            }
        };

        match map.insert(name, args) {
            Some(_) => {
                return Err(name_span
                    .unstable()
                    .error("a binding attribute with the same name already exists"));
            }
            None => {}
        };
    }

    Ok(map)
}

pub fn func_attr_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut target: ItemFn = match parse(input.clone()) {
        Ok(f) => f,
        _ => {
            Span::call_site()
                .unstable()
                .error("the 'func' attribute can only be used on functions")
                .emit();
            return input;
        }
    };

    match validate_function(&target) {
        Ok(_) => {}
        Err(e) => {
            e.emit();
            return input;
        }
    }

    let mut func = match Function::try_from(args) {
        Ok(f) => f,
        Err(e) => {
            e.emit();
            return input;
        }
    };

    let target_name = target.ident.to_string();
    if func.name.is_empty() {
        func.name = target_name.clone();
    }

    func.callback = Some(Ident::new(
        &format!("__invoke_{}", target_name),
        Span::call_site(),
    ));

    let mut binding_args = match drain_binding_attributes(&mut target.attrs) {
        Ok(map) => map,
        Err(e) => {
            e.emit();
            return input;
        }
    };

    let mut has_trigger = false;
    for arg in &target.decl.inputs {
        match bind_argument(&arg, has_trigger, &mut binding_args) {
            Ok(binding) => {
                has_trigger |= binding.is_trigger();
                func.bindings.push(binding);
            }
            Err(e) => {
                e.emit();
                return input;
            }
        };
    }

    if !has_trigger {
        target
            .ident
            .span()
            .unstable()
            .error("Azure Functions must have exactly one trigger input binding")
            .emit();
        return input;
    }

    match bind_return_type(&target.decl.output, &mut binding_args) {
        Ok(Some(binding)) => {
            func.bindings.push(binding);
        }
        Ok(None) => {}
        Err(e) => {
            e.emit();
            return input;
        }
    };

    if let Some((_, args)) = binding_args.iter().nth(0) {
        let (_, value) = args
            .0
            .iter()
            .find(|(k, _)| k.to_string() == "name")
            .unwrap();
        match value {
            Lit::Str(s) => {
                value
                    .span()
                    .unstable()
                    .error(match s.value().as_ref() {
                        RETURN_BINDING_NAME => {
                            "cannot bind to a function without a return value".to_string()
                        }
                        v @ _ => format!(
                            "cannot bind to '{}' because it is not a binding parameter of the function",
                            v
                        ),
                    })
                    .emit();
                return input;
            }
            _ => panic!("expected a string literal for the 'name' argument"),
        }
    }

    let const_name = Ident::new(
        &format!("__{}_FUNCTION", target_name.to_uppercase()),
        Span::call_site(),
    );

    let invoker = TargetInvoker(&target);

    let expanded = quote!{
        #target

        #invoker

        #[allow(dead_code)]
        pub const #const_name: ::azure_functions::codegen::Function = #func;
    };

    expanded.into()
}
