mod binding;
pub mod bindings;
mod function;
mod invoker;
mod output_bindings;

pub use self::binding::*;
pub use self::function::*;
pub use self::invoker::*;
pub use self::output_bindings::*;

use crate::util::{last_segment_in_path, path_to_string, AttributeArguments};
use azure_functions_shared::codegen;
use proc_macro::{Diagnostic, TokenStream};
use proc_macro2::Span;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use syn::spanned::Spanned;
use syn::{
    parse, token::Mut, Attribute, FnArg, GenericArgument, Ident, ItemFn, Lit, Pat, PathArguments,
    PathSegment, ReturnType, Type, Visibility,
};

pub const OUTPUT_BINDING_PREFIX: &str = "output";
const RETURN_BINDING_NAME: &str = "$return";
const CONTEXT_TYPE_NAME: &str = "Context";

fn validate_function(func: &ItemFn) -> Result<(), Diagnostic> {
    match func.vis {
        Visibility::Public(_) => {}
        _ => {
            return Err(func
                .decl
                .fn_token
                .span()
                .unstable()
                .error("the 'func' attribute can only be used on public functions"));
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
            .span
            .unstable()
            .error("the 'func' attribute cannot be used on const functions"));
    }

    if func.unsafety.is_some() {
        return Err(func
            .unsafety
            .as_ref()
            .unwrap()
            .span
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

fn bind_input_type(
    pattern: &Pat,
    ty: &Type,
    mutability: Option<Mut>,
    has_trigger: bool,
    binding_args: &mut HashMap<String, AttributeArguments>,
) -> Result<codegen::Binding, Diagnostic> {
    match ty {
        Type::Path(tp) => {
            let type_name = last_segment_in_path(&tp.path).ident.to_string();

            if type_name == CONTEXT_TYPE_NAME {
                return Ok(codegen::Binding::Context);
            }

            // Check for multiple triggers
            if has_trigger && TRIGGERS.contains_key(type_name.as_str()) {
                return Err(tp
                    .span()
                    .unstable()
                    .error("Azure Functions can only have one trigger binding"));
            }

            // If the reference is mutable, only accept input-output bindings
            let factory = match mutability {
                Some(m) => match INPUT_OUTPUT_BINDINGS.get(type_name.as_str()) {
                    Some(factory) => Ok(factory),
                    None => Err(m.span().unstable().error(
                        "only Azure Functions binding types that support the 'inout' direction can be passed by mutable reference",
                    )),
                },
                None => match TRIGGERS.get(type_name.as_str()) {
                    Some(factory) => Ok(factory),
                    None => match INPUT_BINDINGS.get(type_name.as_str()) {
                        Some(factory) => Ok(factory),
                        None => Err(tp.span().unstable().error(
                            "expected an Azure Functions trigger or input binding type",
                        )),
                    },
                },
            }?;

            match pattern {
                Pat::Ident(name) => {
                    let name_str = name.ident.to_string();
                    match binding_args.remove(&name_str) {
                        Some(args) => (*factory)(args),
                        None => {
                            (*factory)(AttributeArguments::with_name(&name_str, name.ident.span()))
                        }
                    }
                }
                _ => Err(pattern
                    .span()
                    .unstable()
                    .error("bindings must have a named identifier")),
            }
        }
        Type::Paren(tp) => {
            bind_input_type(pattern, &tp.elem, mutability, has_trigger, binding_args)
        }
        _ => Err(ty
            .span()
            .unstable()
            .error("expected an Azure Functions trigger or input binding type")),
    }
}

fn bind_argument(
    arg: &FnArg,
    has_trigger: bool,
    binding_args: &mut HashMap<String, AttributeArguments>,
) -> Result<codegen::Binding, Diagnostic> {
    match arg {
        FnArg::Captured(arg) => match &arg.ty {
            Type::Reference(r) => {
                bind_input_type(&arg.pat, &r.elem, r.mutability, has_trigger, binding_args)
            }
            _ => Err(arg.ty.span().unstable().error(
                "expected an Azure Functions trigger or input binding type passed by reference",
            )),
        },
        FnArg::SelfRef(_) | FnArg::SelfValue(_) => Err(arg
            .span()
            .unstable()
            .error("Azure Functions cannot have self parameters")),
        FnArg::Inferred(_) => Err(arg
            .span()
            .unstable()
            .error("Azure Functions cannot have inferred parameters")),
        FnArg::Ignored(_) => Err(arg
            .span()
            .unstable()
            .error("Azure Functions cannot have ignored parameters")),
    }
}

fn get_option_type(last: &PathSegment) -> Option<&Type> {
    if last.ident != "Option" {
        return None;
    }

    match &last.arguments {
        PathArguments::AngleBracketed(gen_args) => {
            if gen_args.args.len() != 1 {
                return None;
            }
            match gen_args.args.iter().nth(0) {
                Some(GenericArgument::Type(t)) => Some(t),
                _ => None,
            }
        }
        _ => None,
    }
}

fn bind_output_type(
    ty: &Type,
    name: &str,
    binding_args: &mut HashMap<String, AttributeArguments>,
    check_option: bool,
) -> Result<codegen::Binding, Diagnostic> {
    match ty {
        Type::Path(tp) => {
            let last_segment = last_segment_in_path(&tp.path);

            if check_option {
                if let Some(inner) = get_option_type(last_segment) {
                    return bind_output_type(inner, name, binding_args, false);
                }
            }

            match OUTPUT_BINDINGS.get(last_segment.ident.to_string().as_str()) {
                Some(factory) => match binding_args.remove(name) {
                    Some(args) => (*factory)(args),
                    None => (*factory)(AttributeArguments::with_name(name, tp.span())),
                },
                None => Err(tp
                    .span()
                    .unstable()
                    .error("expected an Azure Functions output binding type")),
            }
        }
        Type::Paren(tp) => bind_output_type(&tp.elem, name, binding_args, check_option),
        _ => Err(ty
            .span()
            .unstable()
            .error("expected an Azure Functions output binding type")),
    }
}

fn bind_return_type(
    ret: &ReturnType,
    binding_args: &mut HashMap<String, AttributeArguments>,
) -> Result<Vec<codegen::Binding>, Diagnostic> {
    match ret {
        ReturnType::Default => Ok(vec![]),
        ReturnType::Type(_, ty) => {
            if let Type::Tuple(tuple) = &**ty {
                let mut bindings = vec![];
                for (i, ty) in tuple.elems.iter().enumerate() {
                    if let Type::Tuple(inner) = ty {
                        if !inner.elems.is_empty() {
                            return Err(ty
                                .span()
                                .unstable()
                                .error("expected an Azure Functions output binding type"));
                        }
                        continue;
                    }
                    if i == 0 {
                        bindings.push(bind_output_type(
                            &ty,
                            RETURN_BINDING_NAME,
                            binding_args,
                            true,
                        )?);
                    } else {
                        bindings.push(bind_output_type(
                            &ty,
                            &format!("{}{}", OUTPUT_BINDING_PREFIX, i),
                            binding_args,
                            true,
                        )?);
                    }
                }
                Ok(bindings)
            } else {
                Ok(vec![bind_output_type(
                    &ty,
                    RETURN_BINDING_NAME,
                    binding_args,
                    true,
                )?])
            }
        }
    }
}

fn drain_binding_attributes(
    attrs: &mut Vec<Attribute>,
) -> Result<HashMap<String, AttributeArguments>, Diagnostic> {
    let mut map = HashMap::new();
    for attr in attrs.drain_filter(|a| path_to_string(&a.path) == "binding") {
        let attr_span = attr.span();
        let args = AttributeArguments::try_from(attr)?;

        let (name, name_span) = match args.list.iter().find(|(k, _)| k == "name") {
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

        if map.insert(name, args).is_some() {
            return Err(name_span
                .unstable()
                .error("a binding attribute with the same name already exists"));
        }
    }

    Ok(map)
}

pub fn attr_impl(args: TokenStream, input: TokenStream) -> TokenStream {
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
    };

    let mut func = match Function::try_from(args) {
        Ok(f) => f.0.into_owned(),
        Err(e) => {
            e.emit();
            return input;
        }
    };

    let mut binding_args = match drain_binding_attributes(&mut target.attrs) {
        Ok(map) => map,
        Err(e) => {
            e.emit();
            return input;
        }
    };

    let mut names = HashSet::new();
    let mut has_trigger = false;
    for arg in &target.decl.inputs {
        match bind_argument(&arg, has_trigger, &mut binding_args) {
            Ok(binding) => {
                has_trigger |= binding.is_trigger();

                if let Some(name) = binding.name() {
                    if !names.insert(name.to_string()) {
                        arg.span()
                        .unstable()
                        .error(format!("parameter has camel-cased binding name of '{}' that conflicts with a prior parameter.", name))
                        .emit();
                        return input;
                    }
                }

                func.bindings.to_mut().push(binding);
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
        Ok(bindings) => {
            for binding in bindings.into_iter() {
                if let Some(name) = binding.name() {
                    if !names.insert(name.to_string()) {
                        if let ReturnType::Type(_, ty) = &target.decl.output {
                            ty
                            .span()
                            .unstable()
                            .error(format!("output binding has a name of '{}' that conflicts with a parameter's binding name; the corresponding parameter must be renamed.", name))
                            .emit();
                        }
                        return input;
                    }
                }

                func.bindings.to_mut().push(binding);
            }
        }
        Err(e) => {
            e.emit();
            return input;
        }
    };

    if let Some((_, args)) = binding_args.iter().nth(0) {
        let (_, value) = args.list.iter().find(|(k, _)| k == "name").unwrap();
        match value {
            Lit::Str(s) => {
                value
                    .span()
                    .unstable()
                    .error(match s.value().as_ref() {
                        RETURN_BINDING_NAME => {
                            "cannot bind to a function without a return value".to_string()
                        }
                        v => format!(
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

    let invoker = Invoker(&target);

    let target_name = target.ident.to_string();
    if func.name.is_empty() {
        func.name = Cow::Owned(target_name.clone());
    }

    func.invoker_name = Some(Cow::Owned(invoker.name()));

    let const_name = Ident::new(
        &format!("__{}_FUNCTION", target_name.to_uppercase()),
        Span::call_site(),
    );

    let func = Function(Cow::Borrowed(&func));

    let expanded = quote! {
        #target

        #invoker

        #[allow(dead_code)]
        pub const #const_name: ::azure_functions::codegen::Function = #func;
    };

    expanded.into()
}
