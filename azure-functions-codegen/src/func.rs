mod invoker;
mod output_bindings;

use crate::util::{last_segment_in_path, path_to_string, MacroError};
use azure_functions_shared::codegen::{
    bindings::{Binding, INPUT_BINDINGS, INPUT_OUTPUT_BINDINGS, OUTPUT_BINDINGS, TRIGGERS},
    AttributeArguments, Function, TryFrom,
};
use invoker::Invoker;
use output_bindings::OutputBindings;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use syn::spanned::Spanned;
use syn::{
    parse, token::Mut, Attribute, FnArg, GenericArgument, Ident, ItemFn, Lit, Pat, PathArguments,
    PathSegment, ReturnType, Type, Visibility,
};

pub const OUTPUT_BINDING_PREFIX: &str = "output";
const RETURN_BINDING_NAME: &str = "$return";
const CONTEXT_TYPE_NAME: &str = "Context";

fn validate_function(func: &ItemFn) -> Result<(), MacroError> {
    match func.vis {
        Visibility::Public(_) => {}
        _ => {
            return Err((
                func.decl.fn_token.span(),
                "the 'func' attribute can only be used on public functions".to_string(),
            )
                .into());
        }
    };

    if func.abi.is_some() {
        return Err((
            func.abi.as_ref().unwrap().extern_token.span(),
            "the 'func' attribute cannot be used on extern \"C\" functions".to_string(),
        )
            .into());
    }

    if func.constness.is_some() {
        return Err((
            func.constness.as_ref().unwrap().span,
            "the 'func' attribute cannot be used on const functions".to_string(),
        )
            .into());
    }

    if func.unsafety.is_some() {
        return Err((
            func.unsafety.as_ref().unwrap().span,
            "the 'func' attribute cannot be used on unsafe functions".to_string(),
        )
            .into());
    }

    if !func.decl.generics.params.is_empty() {
        return Err((
            func.decl.generics.params.span(),
            "the 'func' attribute cannot be used on generic functions".to_string(),
        )
            .into());
    }

    if func.decl.variadic.is_some() {
        return Err((
            func.decl.variadic.span(),
            "the 'func' attribute cannot be used on variadic functions".to_string(),
        )
            .into());
    }

    Ok(())
}

fn bind_input_type(
    pattern: &Pat,
    ty: &Type,
    mutability: Option<Mut>,
    has_trigger: bool,
    binding_args: &mut HashMap<String, AttributeArguments>,
) -> Result<Binding, MacroError> {
    match ty {
        Type::Path(tp) => {
            let type_name = last_segment_in_path(&tp.path).ident.to_string();

            if type_name == CONTEXT_TYPE_NAME {
                return Ok(Binding::Context);
            }

            // Check for multiple triggers
            if has_trigger && TRIGGERS.contains_key(type_name.as_str()) {
                return Err((
                    tp.span(),
                    "Azure Functions can only have one trigger binding".to_string(),
                )
                    .into());
            }

            // If the reference is mutable, only accept input-output bindings
            let factory = match mutability {
                Some(m) => match INPUT_OUTPUT_BINDINGS.get(type_name.as_str()) {
                    Some(factory) => Ok(factory),
                    None => Err((m.span(),
                        "only Azure Functions binding types that support the 'inout' direction can be passed by mutable reference".to_string(),
                    ).into()),
                },
                None => match TRIGGERS.get(type_name.as_str()) {
                    Some(factory) => Ok(factory),
                    None => match INPUT_BINDINGS.get(type_name.as_str()) {
                        Some(factory) => Ok(factory),
                        None => Err((tp.span(),
                            "expected an Azure Functions trigger or input binding type".to_string(),
                        ).into()),
                    },
                },
            }?;

            match pattern {
                Pat::Ident(name) => {
                    let name_str = name.ident.to_string();
                    match binding_args.remove(&name_str) {
                        Some(args) => (*factory)(args).map_err(|e| e.into()),
                        None => {
                            (*factory)(AttributeArguments::with_name(&name_str, name.ident.span()))
                                .map_err(|e| e.into())
                        }
                    }
                }
                _ => Err((
                    pattern.span(),
                    "bindings must have a named identifier".to_string(),
                )
                    .into()),
            }
        }
        Type::Paren(tp) => {
            bind_input_type(pattern, &tp.elem, mutability, has_trigger, binding_args)
        }
        _ => Err((
            ty.span(),
            "expected an Azure Functions trigger or input binding type".to_string(),
        )
            .into()),
    }
}

fn bind_argument(
    arg: &FnArg,
    has_trigger: bool,
    binding_args: &mut HashMap<String, AttributeArguments>,
) -> Result<Binding, MacroError> {
    match arg {
        FnArg::Captured(arg) => match &arg.ty {
            Type::Reference(r) => {
                bind_input_type(&arg.pat, &r.elem, r.mutability, has_trigger, binding_args)
            }
            _ => Err((
                arg.ty.span(),
                "expected an Azure Functions trigger or input binding type passed by reference"
                    .to_string(),
            )
                .into()),
        },
        FnArg::SelfRef(_) | FnArg::SelfValue(_) => Err((
            arg.span(),
            "Azure Functions cannot have self parameters".to_string(),
        )
            .into()),
        FnArg::Inferred(_) => Err((
            arg.span(),
            "Azure Functions cannot have inferred parameters".to_string(),
        )
            .into()),
        FnArg::Ignored(_) => Err((
            arg.span(),
            "Azure Functions cannot have ignored parameters".to_string(),
        )
            .into()),
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
) -> Result<Binding, MacroError> {
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
                    Some(args) => (*factory)(args).map_err(|e| e.into()),
                    None => (*factory)(AttributeArguments::with_name(name, tp.span()))
                        .map_err(|e| e.into()),
                },
                None => Err((
                    tp.span(),
                    "expected an Azure Functions output binding type".to_string(),
                )
                    .into()),
            }
        }
        Type::Paren(tp) => bind_output_type(&tp.elem, name, binding_args, check_option),
        _ => Err((
            ty.span(),
            "expected an Azure Functions output binding type".to_string(),
        )
            .into()),
    }
}

fn bind_return_type(
    ret: &ReturnType,
    binding_args: &mut HashMap<String, AttributeArguments>,
) -> Result<Vec<Binding>, MacroError> {
    match ret {
        ReturnType::Default => Ok(vec![]),
        ReturnType::Type(_, ty) => {
            if let Type::Tuple(tuple) = &**ty {
                let mut bindings = vec![];
                for (i, ty) in tuple.elems.iter().enumerate() {
                    if let Type::Tuple(inner) = ty {
                        if !inner.elems.is_empty() {
                            return Err((
                                ty.span(),
                                "expected an Azure Functions output binding type".to_string(),
                            )
                                .into());
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
) -> Result<HashMap<String, AttributeArguments>, MacroError> {
    let mut map = HashMap::new();
    // TODO: use drain_filter when stable https://github.com/rust-lang/rust/issues/43244
    for attr in attrs
        .iter()
        .filter(|a| path_to_string(&a.path) == "binding")
    {
        let attr_span = attr.span();
        let args = AttributeArguments::try_from(attr.clone()).map_err(|e| e.into())?;

        let (name, name_span) = match args.list.iter().find(|(k, _)| k == "name") {
            Some((_, v)) => match v {
                Lit::Str(s) => (s.value(), s.span()),
                _ => {
                    return Err((
                        v.span(),
                        "expected a literal string value for the 'name' argument".to_string(),
                    )
                        .into());
                }
            },
            None => {
                return Err((
                    attr_span,
                    "binding attributes must have a 'name' argument".to_string(),
                )
                    .into());
            }
        };

        if map.insert(name, args).is_some() {
            return Err((
                name_span,
                "a binding attribute with the same name already exists".to_string(),
            )
                .into());
        }
    }

    attrs.retain(|a| path_to_string(&a.path) != "binding");

    Ok(map)
}

pub fn attr_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut target: ItemFn = match parse(input.clone()) {
        Ok(f) => f,
        _ => {
            let error: MacroError = (
                Span::call_site(),
                "the 'func' attribute can only be used on functions".to_string(),
            )
                .into();
            error.emit();
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

    let mut func = match Function::try_from(proc_macro2::TokenStream::from(args)) {
        Ok(f) => f,
        Err(e) => {
            let e: MacroError = e.into();
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
                        let error: MacroError = (arg.span(),
                            format!("parameter has camel-cased binding name of '{}' that conflicts with a prior parameter.", name)).into();
                        error.emit();
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
        let error: MacroError = (
            target.ident.span(),
            "Azure Functions must have exactly one trigger input binding".to_string(),
        )
            .into();
        error.emit();
        return input;
    }

    match bind_return_type(&target.decl.output, &mut binding_args) {
        Ok(bindings) => {
            for binding in bindings.into_iter() {
                if let Some(name) = binding.name() {
                    if !names.insert(name.to_string()) {
                        if let ReturnType::Type(_, ty) = &target.decl.output {
                            let error: MacroError = (ty
                            .span(),
                            format!("output binding has a name of '{}' that conflicts with a parameter's binding name; the corresponding parameter must be renamed.", name)).into();
                            error.emit();
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
                let message = match s.value().as_ref() {
                    RETURN_BINDING_NAME => {
                        "cannot bind to a function without a return value".into()
                    }
                    v => format!(
                        "cannot bind to '{}' because it is not a binding parameter of the function",
                        v
                    ),
                };
                let error: MacroError = (value.span(), message).into();
                error.emit();
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

    let expanded = quote! {
        #target

        #invoker

        #[allow(dead_code)]
        pub const #const_name: ::azure_functions::codegen::Function = #func;
    };

    expanded.into()
}
