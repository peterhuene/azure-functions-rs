mod invoker;
mod output_bindings;

use crate::{attribute_args_from_name, parse_attribute_args};
use azure_functions_shared::codegen::{
    bindings::{
        Binding, BindingFactory, INPUT_BINDINGS, INPUT_OUTPUT_BINDINGS, OUTPUT_BINDINGS, TRIGGERS,
        VEC_INPUT_BINDINGS, VEC_OUTPUT_BINDINGS,
    },
    get_string_value, iter_attribute_args, last_segment_in_path, macro_panic, Function, InvokerFn,
};
use invoker::Invoker;
use output_bindings::OutputBindings;
use proc_macro2::Span;
use quote::quote;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use syn::spanned::Spanned;
use syn::{
    parse, token::Mut, Attribute, AttributeArgs, FnArg, GenericArgument, Ident, ItemFn, Lit, Pat,
    PathArguments, PathSegment, ReturnType, Type, TypePath, Visibility,
};

pub const OUTPUT_BINDING_PREFIX: &str = "output";
const RETURN_BINDING_NAME: &str = "$return";
const ORCHESTRATION_CONTEXT_TYPE: &str = "DurableOrchestrationContext";
const ORCHESTRATION_OUTPUT_TYPE: &str = "OrchestrationOutput";
const ACTIVITY_CONTEXT_TYPE: &str = "DurableActivityContext";
const ACTIVITY_OUTPUT_TYPE: &str = "ActivityOutput";

fn has_parameter_of_type(func: &ItemFn, type_name: &str) -> bool {
    func.sig.inputs.iter().any(|arg| {
        if let FnArg::Typed(arg) = arg {
            match &*arg.ty {
                Type::Reference(tr) => {
                    if let Type::Path(tp) = &*tr.elem {
                        return last_segment_in_path(&tp.path).ident == type_name;
                    }
                }
                Type::Path(tp) => {
                    return last_segment_in_path(&tp.path).ident == type_name;
                }
                _ => {}
            }
        }
        false
    })
}

fn validate_orchestration_function(func: &ItemFn) {
    if func.sig.asyncness.is_none() {
        macro_panic(
            func.sig.ident.span(),
            "orchestration functions must be async",
        );
    }

    if func.sig.inputs.len() != 1 {
        macro_panic(
            func.sig.ident.span(),
            format!(
                "orchestration functions must have exactly one parameter of type `{}`",
                ORCHESTRATION_CONTEXT_TYPE
            ),
        );
    }

    if !match func.sig.inputs.iter().next().unwrap() {
        FnArg::Typed(arg) => match &*arg.ty {
            Type::Path(tp) => last_segment_in_path(&tp.path).ident == ORCHESTRATION_CONTEXT_TYPE,
            _ => false,
        },
        _ => false,
    } {
        macro_panic(
            func.sig.ident.span(),
            format!(
                "orchestration functions must have exactly one parameter of type `{}`",
                ORCHESTRATION_CONTEXT_TYPE
            ),
        );
    }

    if let ReturnType::Type(_, ty) = &func.sig.output {
        match ty.as_ref() {
            Type::Path(tp) => {
                if last_segment_in_path(&tp.path).ident != ORCHESTRATION_OUTPUT_TYPE {
                    macro_panic(
                        tp.span(),
                        format!(
                            "orchestration functions must have a return type of `{}`",
                            ORCHESTRATION_OUTPUT_TYPE
                        ),
                    );
                }
            }
            _ => macro_panic(
                ty.span(),
                format!(
                    "orchestration functions must have a return type of `{}`",
                    ORCHESTRATION_OUTPUT_TYPE
                ),
            ),
        }
    }
}

fn validate_activity_function(func: &ItemFn) {
    // Activity functions cannot have a $return binding
    // Default, -> ActivityOutput, and -> (ActivityOutput, ...) are acceptable

    fn validate_return_binding(ty: &Type) {
        match ty {
            Type::Tuple(tuple) => {
                if let Some(first) = tuple.elems.iter().next() {
                    validate_return_binding(first)
                }
            }
            Type::Paren(tp) => validate_return_binding(&*tp.elem),
            Type::Path(tp) => {
                if last_segment_in_path(&tp.path).ident != ACTIVITY_OUTPUT_TYPE {
                    macro_panic(
                        tp.span(),
                        format!(
                            "activity functions must have a return type of `{}`",
                            ACTIVITY_OUTPUT_TYPE
                        ),
                    );
                }
            }
            _ => macro_panic(
                ty.span(),
                format!(
                    "activity functions must have a return type of `{}`",
                    ACTIVITY_OUTPUT_TYPE
                ),
            ),
        }
    }

    if let ReturnType::Type(_, ty) = &func.sig.output {
        validate_return_binding(&*ty);
    }
}

fn validate_function(func: &ItemFn) {
    match func.vis {
        Visibility::Public(_) => {}
        _ => macro_panic(
            func.sig.fn_token.span(),
            "the 'func' attribute can only be used on public functions",
        ),
    };

    if func.sig.abi.is_some() {
        macro_panic(
            func.sig.abi.as_ref().unwrap().extern_token.span(),
            "the 'func' attribute cannot be used on extern \"C\" functions",
        );
    }

    if func.sig.constness.is_some() {
        macro_panic(
            func.sig.constness.as_ref().unwrap().span,
            "the 'func' attribute cannot be used on const functions",
        );
    }

    if func.sig.unsafety.is_some() {
        macro_panic(
            func.sig.unsafety.as_ref().unwrap().span,
            "the 'func' attribute cannot be used on unsafe functions",
        );
    }

    if !func.sig.generics.params.is_empty() {
        macro_panic(
            func.sig.generics.params.span(),
            "the 'func' attribute cannot be used on generic functions",
        );
    }

    if func.sig.variadic.is_some() {
        macro_panic(
            func.sig.variadic.span(),
            "the 'func' attribute cannot be used on variadic functions",
        );
    }
}

fn get_generic_argument_type<'a>(
    last: &'a PathSegment,
    generic_type_name: &str,
) -> Option<&'a Type> {
    if last.ident != generic_type_name {
        return None;
    }

    match &last.arguments {
        PathArguments::AngleBracketed(gen_args) => {
            if gen_args.args.len() != 1 {
                return None;
            }
            match gen_args.args.iter().next() {
                Some(GenericArgument::Type(t)) => Some(t),
                _ => None,
            }
        }
        _ => None,
    }
}

fn get_input_binding_factory_for_vec(tp: &TypePath, mutability: Option<Mut>) -> &BindingFactory {
    let last_segment = last_segment_in_path(&tp.path);
    let type_name = last_segment.ident.to_string();

    if let Some(mutability) = mutability {
        macro_panic(
            mutability.span(),
            "vector bindings cannot be passed by mutable reference",
        );
    }

    if !VEC_INPUT_BINDINGS.contains(type_name.as_str()) {
        macro_panic(
            tp.span(),
            format!(
                "`Vec<{}>` is not a supported Azure Function input binding type",
                type_name
            ),
        );
    }

    match INPUT_BINDINGS.get(type_name.as_str()) {
        Some(factory) => factory,
        None => macro_panic(
            tp.span(),
            format!(
                "{} is not a supported Azure Function input binding type",
                type_name
            ),
        ),
    }
}

fn get_output_binding_factory(tp: &TypePath) -> &BindingFactory {
    let last_segment = last_segment_in_path(&tp.path);
    let mut type_name = last_segment.ident.to_string();

    if let Some(ty) = get_generic_argument_type(last_segment_in_path(&tp.path), "Vec") {
        match ty {
            Type::Path(tp) => {
                type_name = last_segment_in_path(&tp.path).ident.to_string();
                if !VEC_OUTPUT_BINDINGS.contains(type_name.as_str()) {
                    macro_panic(
                        tp.span(),
                        format!(
                            "`Vec<{}>` is not a supported Azure Function output binding type",
                            type_name
                        ),
                    );
                }
            }
            _ => macro_panic(
                ty.span(),
                format!(
                    "{} is not a supported Azure Function output binding type",
                    type_name
                ),
            ),
        }
    }

    match OUTPUT_BINDINGS.get(type_name.as_str()) {
        Some(factory) => factory,
        None => macro_panic(
            tp.span(),
            format!(
                "{} is not a supported Azure Function output binding type",
                type_name
            ),
        ),
    }
}

fn get_input_binding_factory(
    tp: &TypePath,
    mutability: Option<Mut>,
    has_trigger: bool,
) -> &BindingFactory {
    let last_segment = last_segment_in_path(&tp.path);

    if let Some(ty) = get_generic_argument_type(&last_segment, "Vec") {
        match ty {
            Type::Path(tp) => {
                return get_input_binding_factory_for_vec(tp, mutability);
            }
            _ => macro_panic(ty.span(), "expected an Azure Function input binding type"),
        }
    }

    // Check for multiple triggers
    let type_name = last_segment.ident.to_string();
    if has_trigger && TRIGGERS.contains_key(type_name.as_str()) {
        macro_panic(
            tp.span(),
            "Azure Functions can only have one trigger binding",
        );
    }

    match mutability {
        Some(m) => match INPUT_OUTPUT_BINDINGS.get(type_name.as_str()) {
            Some(factory) => factory,
            None => macro_panic(
                m.span(),
                format!(
                    "{} is not a supported Azure Function inout binding type",
                    type_name
                ),
            ),
        },
        None => match TRIGGERS.get(type_name.as_str()) {
            Some(factory) => factory,
            None => match INPUT_BINDINGS.get(type_name.as_str()) {
                Some(factory) => factory,
                None => macro_panic(
                    tp.span(),
                    format!(
                        "{} is not a supported Azure Function trigger or input binding type",
                        type_name
                    ),
                ),
            },
        },
    }
}

fn bind_input_type(
    pattern: &Pat,
    tp: &TypePath,
    mutability: Option<Mut>,
    has_trigger: bool,
    binding_args: &mut HashMap<String, (AttributeArgs, Span)>,
) -> Binding {
    let factory = get_input_binding_factory(tp, mutability, has_trigger);

    match pattern {
        Pat::Ident(name) => {
            let name_str = name.ident.to_string();
            match binding_args.remove(&name_str) {
                Some(args) => (*factory)(args.0, args.1),
                None => {
                    let name_span = name.ident.span();
                    (*factory)(attribute_args_from_name(&name_str, name_span), name_span)
                }
            }
        }
        _ => macro_panic(pattern.span(), "bindings must have a named identifier"),
    }
}

fn bind_argument(
    arg: &FnArg,
    has_trigger: bool,
    binding_args: &mut HashMap<String, (AttributeArgs, Span)>,
) -> Binding {
    match arg {
        FnArg::Typed(arg) => match &*arg.ty {
            Type::Reference(tr) => match &*tr.elem {
                Type::Path(tp) => {
                    bind_input_type(&*arg.pat, tp, tr.mutability, has_trigger, binding_args)
                }
                _ => macro_panic(
                    arg.ty.span(),
                    "expected an Azure Functions trigger or input binding type",
                ),
            },
            Type::Path(tp) => bind_input_type(&*arg.pat, tp, None, has_trigger, binding_args),
            _ => macro_panic(
                arg.ty.span(),
                "expected an Azure Functions trigger or input binding type",
            ),
        },
        FnArg::Receiver(_) => {
            macro_panic(arg.span(), "Azure Functions cannot have self parameters")
        }
    }
}

fn bind_output_type(
    ty: &Type,
    name: &str,
    binding_args: &mut HashMap<String, (AttributeArgs, Span)>,
    check_option: bool,
) -> Binding {
    match ty {
        Type::Path(tp) => {
            let last_segment = last_segment_in_path(&tp.path);

            if check_option {
                if let Some(inner) = get_generic_argument_type(last_segment, "Option") {
                    return bind_output_type(inner, name, binding_args, false);
                }
            }

            let factory = get_output_binding_factory(tp);

            match binding_args.remove(name) {
                Some(args) => (*factory)(args.0, args.1),
                None => {
                    let span = tp.span();
                    (*factory)(attribute_args_from_name(name, span), span)
                }
            }
        }
        Type::Paren(tp) => bind_output_type(&tp.elem, name, binding_args, check_option),
        _ => macro_panic(ty.span(), "expected an Azure Functions output binding type"),
    }
}

fn bind_return_type(
    ret: &ReturnType,
    binding_args: &mut HashMap<String, (AttributeArgs, Span)>,
    is_activity: bool,
) -> Vec<Binding> {
    let mut bindings = Vec::new();

    if let ReturnType::Type(_, ty) = ret {
        if let Type::Tuple(tuple) = &**ty {
            for (i, ty) in tuple.elems.iter().enumerate() {
                if let Type::Tuple(inner) = ty {
                    if !inner.elems.is_empty() {
                        macro_panic(ty.span(), "expected an Azure Functions output binding type");
                    }
                    continue;
                }
                if i == 0 {
                    if !is_activity {
                        bindings.push(bind_output_type(
                            &ty,
                            RETURN_BINDING_NAME,
                            binding_args,
                            true,
                        ));
                    }
                } else {
                    bindings.push(bind_output_type(
                        &ty,
                        &format!("{}{}", OUTPUT_BINDING_PREFIX, i),
                        binding_args,
                        true,
                    ));
                }
            }
        } else if !is_activity {
            bindings.push(bind_output_type(
                &ty,
                RETURN_BINDING_NAME,
                binding_args,
                true,
            ));
        }
    }

    bindings
}

fn drain_binding_attributes(attrs: &mut Vec<Attribute>) -> HashMap<String, (AttributeArgs, Span)> {
    let mut map = HashMap::new();
    // TODO: use drain_filter when stable https://github.com/rust-lang/rust/issues/43244
    for attr in attrs
        .iter()
        .filter(|a| last_segment_in_path(&a.path).ident == "binding")
    {
        let attr_span = attr.span();
        let args = parse_attribute_args(&attr);
        let mut name = None;
        let mut name_span = None;

        iter_attribute_args(&args, |key, value| {
            if key != "name" {
                return true;
            }

            name = Some(get_string_value("name", value));
            name_span = Some(key.span());
            false
        });

        if name.is_none() {
            macro_panic(attr_span, "binding attributes must have a 'name' argument");
        }

        if map.insert(name.unwrap(), (args, attr.span())).is_some() {
            macro_panic(attr_span, "binding attributes must have a 'name' argument");
        }
    }

    attrs.retain(|a| last_segment_in_path(&a.path).ident != "binding");

    map
}

pub fn func_impl(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut target: ItemFn = match parse(input) {
        Ok(f) => f,
        _ => macro_panic(
            Span::call_site(),
            "the 'func' attribute can only be used on functions",
        ),
    };

    validate_function(&target);

    let is_orchestration = has_parameter_of_type(&target, ORCHESTRATION_CONTEXT_TYPE);
    let is_activity = has_parameter_of_type(&target, ACTIVITY_CONTEXT_TYPE);

    if is_orchestration {
        validate_orchestration_function(&target);
    } else if is_activity {
        validate_activity_function(&target);
    }

    let mut func = Function::from(match syn::parse_macro_input::parse::<AttributeArgs>(args) {
        Ok(f) => f,
        Err(e) => macro_panic(
            Span::call_site(),
            format!("failed to parse attribute arguments: {}", e),
        ),
    });

    let mut binding_args = drain_binding_attributes(&mut target.attrs);
    let mut names = HashSet::new();
    let mut has_trigger = false;
    for arg in &target.sig.inputs {
        let binding = bind_argument(&arg, has_trigger, &mut binding_args);
        has_trigger |= binding.is_trigger();

        if let Some(name) = binding.name() {
            if !names.insert(name.to_string()) {
                macro_panic(arg.span(), format!("parameter has camel-cased binding name of '{}' that conflicts with a prior parameter.", name));
            }
        }

        func.bindings.to_mut().push(binding);
    }

    if !has_trigger {
        macro_panic(
            target.sig.ident.span(),
            "Azure Functions must have exactly one trigger input binding",
        );
    }

    if !is_orchestration {
        for binding in
            bind_return_type(&target.sig.output, &mut binding_args, is_activity).into_iter()
        {
            if let Some(name) = binding.name() {
                if !names.insert(name.to_string()) {
                    if let ReturnType::Type(_, ty) = &target.sig.output {
                        macro_panic(ty.span(), format!("output binding has a name of '{}' that conflicts with a parameter's binding name; the corresponding parameter must be renamed.", name));
                    }
                    macro_panic(target.sig.output.span(), format!("output binding has a name of '{}' that conflicts with a parameter's binding name; the corresponding parameter must be renamed.", name));
                }
            }

            func.bindings.to_mut().push(binding);
        }
    }

    if let Some((_, args)) = binding_args.iter().next() {
        iter_attribute_args(&args.0, |k, v| {
            if k != "name" {
                return true;
            }

            if let Lit::Str(s) = v {
                match s.value().as_ref() {
                    RETURN_BINDING_NAME => if is_orchestration {
                        macro_panic(
                            v.span(),
                            "cannot bind to the return value of an orchestration function",
                        )
                    } else if is_activity {
                        macro_panic(
                            v.span(),
                            "cannot bind to the return value of an activity function",
                        )
                    } else {
                        macro_panic(
                            v.span(),
                            "cannot bind to a function without a return value",
                        )
                    },
                    v => macro_panic(
                        v.span(),
                        format!(
                            "cannot bind to '{}' because it is not a binding parameter of the function",
                            v
                        ),
                    ),
                };
            } else {
                macro_panic(
                    v.span(),
                    "expected a string literal for the 'name' argument",
                );
            }
        });
    }

    let invoker = Invoker {
        func: &target,
        is_orchestration,
    };

    let target_name = target.sig.ident.to_string();
    if func.name.is_empty() {
        func.name = Cow::Owned(target_name.clone());
    }

    if !is_orchestration && target.sig.asyncness.is_some() {
        func.invoker = Some(azure_functions_shared::codegen::Invoker {
            name: Cow::Owned(invoker.name()),
            invoker_fn: InvokerFn::Async(None),
        });
    } else {
        func.invoker = Some(azure_functions_shared::codegen::Invoker {
            name: Cow::Owned(invoker.name()),
            invoker_fn: InvokerFn::Sync(None),
        });
    }

    let const_name = Ident::new(
        &format!("{}_FUNCTION", target_name.to_uppercase()),
        Span::call_site(),
    );

    quote!(
        #target

        #invoker

        #[allow(dead_code)]
        pub const #const_name: ::azure_functions::codegen::Function = #func;
    )
    .into()
}
