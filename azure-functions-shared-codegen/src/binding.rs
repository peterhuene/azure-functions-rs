use crate::{iter_attribute_args, last_segment_in_path, macro_panic, parse_attribute_args};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse, spanned::Spanned, AttributeArgs, Fields, Ident, ItemStruct, Lit, Type, TypePath};

fn get_string_value(name: &str, value: &Lit) -> String {
    if let Lit::Str(s) = value {
        return s.value();
    }

    macro_panic(
        value.span(),
        format!(
            "expected a literal string value for the '{}' argument",
            name
        ),
    )
}

fn get_boolean_value(name: &str, value: &Lit) -> bool {
    if let Lit::Bool(b) = value {
        return b.value;
    }

    macro_panic(
        value.span(),
        format!(
            "expected a literal boolean value for the '{}' argument",
            name
        ),
    )
}

struct BindingArguments {
    name: String,
    direction: Option<String>,
    validate: Option<String>,
}

impl BindingArguments {
    fn get_validation_call(&self) -> TokenStream {
        if let Some(validate) = self.validate.as_ref() {
            let ident = Ident::new(validate, Span::call_site());
            quote!(
                if let Err(message) = __binding.#ident() {
                    crate::codegen::macro_panic(__args_and_span.1, message);
                }
            )
        } else {
            quote!()
        }
    }
}

impl From<AttributeArgs> for BindingArguments {
    fn from(args: AttributeArgs) -> Self {
        let mut name = None;
        let mut direction = None;
        let mut validate = None;

        iter_attribute_args(&args, |key, value| {
            let key_name = key.to_string();

            match key_name.as_ref() {
                "name" => name = Some(get_string_value("name", value)),
                "direction" => direction = Some(get_string_value("direction", value)),
                "validate" => validate = Some(get_string_value("validate", value)),
                _ => macro_panic(
                    key.span(),
                    format!("unsupported binding attribute argument '{}'", key_name),
                ),
            };

            true
        });

        if name.is_none() {
            macro_panic(
                Span::call_site(),
                "the 'name' argument is required for a binding.",
            );
        }

        BindingArguments {
            name: name.unwrap(),
            direction,
            validate,
        }
    }
}

#[derive(Debug)]
struct FieldArguments {
    name: Option<String>,
    camel_case_value: Option<bool>,
    values: Option<String>,
}

impl From<AttributeArgs> for FieldArguments {
    fn from(args: AttributeArgs) -> Self {
        let mut name = None;
        let mut camel_case_value = None;
        let mut values = None;

        iter_attribute_args(&args, |key, value| {
            let key_name = key.to_string();

            match key_name.as_ref() {
                "name" => name = Some(get_string_value("name", value)),
                "camel_case_value" => {
                    camel_case_value = Some(get_boolean_value("camel_case", value))
                }
                "values" => values = Some(get_string_value("values", value)),
                _ => macro_panic(
                    key.span(),
                    format!("unsupported binding attribute argument '{}'", key_name),
                ),
            };

            true
        });

        FieldArguments {
            name,
            camel_case_value,
            values,
        }
    }
}

fn drain_field_attributes(fields: &mut Fields) {
    if let Fields::Named(fields) = fields {
        for field in fields.named.iter_mut() {
            field
                .attrs
                .retain(|a| last_segment_in_path(&a.path).ident != "field");
        }
    } else {
        macro_panic(fields.span(), "binding structure fields must be named");
    }
}

enum FieldType {
    String,          // Cow<'static, str>,
    OptionalString,  // Option<Cow<'static, str>>,
    Boolean,         // bool,
    OptionalBoolean, // Option<bool>
    Direction,       // Direction,
    StringArray,     // Cow<'static, [Cow<'static, str>]>,
    Integer,         // i64,
    OptionalInteger, // Option<i64>,
}

impl From<&TypePath> for FieldType {
    fn from(tp: &TypePath) -> Self {
        let mut stream = TokenStream::new();
        tp.path.to_tokens(&mut stream);

        let mut type_name = stream.to_string();
        type_name.retain(|c| c != ' ');

        match type_name.as_ref() {
            "Cow<'static,str>" => FieldType::String,
            "Option<Cow<'static,str>>" => FieldType::OptionalString,
            "bool" => FieldType::Boolean,
            "Option<bool>" => FieldType::OptionalBoolean,
            "Direction" => FieldType::Direction,
            "Cow<'static,[Cow<'static,str>]>" => FieldType::StringArray,
            "i64" => FieldType::Integer,
            "Option<i64>" => FieldType::OptionalInteger,
            _ => macro_panic(
                tp.span(),
                format!("field type '{}' is not supported for a binding", type_name),
            ),
        }
    }
}

struct Field {
    ident: Ident,
    args: Option<FieldArguments>,
    ty: FieldType,
}

impl Field {
    pub fn get_serialization(&self) -> TokenStream {
        let ident = &self.ident;

        let mut name = &ident.to_string();

        if let Some(args) = &self.args {
            if let Some(n) = &args.name {
                name = n;
            }
        }

        match &self.ty {
            FieldType::OptionalString | FieldType::OptionalBoolean | FieldType::OptionalInteger => {
                quote!(
                    if let Some(#ident) = self.#ident.as_ref() {
                        map.serialize_entry(#name, #ident)?;
                    }
                )
            }
            FieldType::StringArray => quote!(
                if !self.#ident.is_empty() {
                    map.serialize_entry(#name, &self.#ident)?;
                }
            ),
            _ => quote!(map.serialize_entry(#name, &self.#ident)?;),
        }
    }

    pub fn get_field_decl(&self) -> Option<TokenStream> {
        let ident = &self.ident;
        match &self.ty {
            FieldType::Direction => None,
            _ => Some(quote!(let mut #ident = None;)),
        }
    }

    pub fn get_field_match(&self) -> Option<TokenStream> {
        match &self.ty {
            FieldType::Direction => None,
            _ => {
                let ident = &self.ident;

                let camel_case_value = match &self.args {
                    Some(args) => args.camel_case_value.unwrap_or(false),
                    _ => false,
                };

                match self.ty {
                    FieldType::String | FieldType::OptionalString => {
                        let validation = self.get_field_validation();
                        if camel_case_value {
                            Some(quote!(
                                stringify!(#ident) => {
                                    let __v = crate::util::to_camel_case(&crate::codegen::get_string_value(stringify!(#ident), &__value));
                                    #validation
                                    #ident = Some(Cow::from(__v));
                                }
                            ))
                        } else {
                            Some(quote!(
                                stringify!(#ident) => {
                                    let __v = crate::codegen::get_string_value(stringify!(#ident), &__value);
                                    #validation
                                    #ident = Some(Cow::from(__v));
                                }
                            ))
                        }
                    }
                    FieldType::Boolean | FieldType::OptionalBoolean => Some(
                        quote!(stringify!(#ident) => #ident = Some(crate::codegen::get_boolean_value(stringify!(#ident), &__value))),
                    ),
                    FieldType::Direction => panic!("cannot get a match type for a direction field"),
                    FieldType::StringArray => {
                        let validation = self.get_field_validation();
                        Some(quote!(stringify!(#ident) => {
                                let __v: Vec<_> = crate::codegen::get_string_value(stringify!(#ident), &__value).split('|').map(|v| Cow::from(v.to_string())).collect();
                                #validation
                                #ident = Some(Cow::from(__v));
                            }
                        ))
                    }

                    FieldType::Integer | FieldType::OptionalInteger => Some(
                        quote!(stringify!(#ident) => #ident = Some(crate::codegen::get_integer_value(stringify!(#ident), &__value))),
                    ),
                }
            }
        }
    }

    pub fn get_required_check(&self) -> Option<TokenStream> {
        let ident = &self.ident;
        match &self.ty {
            FieldType::String | FieldType::Boolean | FieldType::Integer => Some(quote!(
                if #ident.is_none() {
                    crate::codegen::macro_panic(__args_and_span.1, concat!("the '", stringify!(#ident), "' argument is required for this binding"));
                }
            )),
            _ => None,
        }
    }

    pub fn get_field_validation(&self) -> TokenStream {
        if let Some(args) = &self.args {
            if let Some(values) = &args.values {
                let ident = &self.ident;
                match self.ty {
                    FieldType::String | FieldType::OptionalString => {
                        return quote!(
                            if !#values.split('|').map(str::trim).any(|v| v == __v.to_lowercase()) {
                                crate::codegen::macro_panic(__key.span(), format!(concat!("'{}' is not a valid value for the '", stringify!(#ident), "' attribute"), __v));
                            }
                        );
                    },
                    FieldType::StringArray => {
                        return quote!(
                            let __acceptable: Vec<&str> = #values.split('|').map(str::trim).collect();
                            for v in __v.iter() {
                                if !__acceptable.contains(&v.as_ref().to_lowercase().as_ref()) {
                                    crate::codegen::macro_panic(__key.span(), format!(concat!("'{}' is not a valid value for the '", stringify!(#ident), "' attribute"), v));
                                }
                            }
                        );
                    },
                    _ => macro_panic(self.ident.span(), "only fields of type string or arrays of string can have a 'values' attribute")
                }
            }
        }
        quote!()
    }

    pub fn get_field_assignment(&self) -> TokenStream {
        let ident = &self.ident;
        match &self.ty {
            FieldType::Direction => quote!(#ident: Default::default()),
            FieldType::OptionalString | FieldType::OptionalBoolean | FieldType::OptionalInteger => quote!(#ident),
            FieldType::StringArray => quote!(#ident: #ident.unwrap_or(Cow::Borrowed(&[]))),
            _ => quote!(#ident: #ident.unwrap()),
        }
    }

    pub fn get_quotable_decl(&self) -> TokenStream {
        let ident = &self.ident;
        match self.ty {
            FieldType::String => quote!(let #ident = crate::codegen::quotable::QuotableBorrowedStr(&self.#ident);),
            FieldType::OptionalString => quote!(let #ident = crate::codegen::quotable::QuotableOption(self.#ident.as_ref().map(|x| crate::codegen::quotable::QuotableBorrowedStr(x)));),
            FieldType::Boolean => quote!(let #ident = self.#ident;),
            FieldType::Direction => quote!(let #ident = crate::codegen::quotable::QuotableDirection(self.#ident);),
            FieldType::StringArray => quote!(let #ident = crate::codegen::quotable::QuotableStrArray(self.#ident.as_ref());),
            FieldType::Integer => quote!(let #ident = self.#ident),
            FieldType::OptionalBoolean | FieldType::OptionalInteger => quote!(let #ident = crate::codegen::quotable::QuotableOption(self.#ident);),
        }
    }

    pub fn get_quotable_assignment(&self) -> TokenStream {
        let ident = &self.ident;
        quote!(#ident: ##ident)
    }
}

impl From<&syn::Field> for Field {
    fn from(field: &syn::Field) -> Self {
        let mut args = None;
        for attr in field
            .attrs
            .iter()
            .filter(|a| last_segment_in_path(&a.path).ident == "field")
        {
            if args.is_some() {
                macro_panic(
                    attr.span(),
                    "a field can only have at most one attribute applied",
                );
            }

            args = Some(parse_attribute_args(&attr));
        }

        Field {
            ident: field.ident.as_ref().unwrap().clone(),
            args: args.map(Into::into),
            ty: match &field.ty {
                Type::Path(tp) => tp.into(),
                _ => panic!("expected a type path for field type"),
            },
        }
    }
}

fn get_default_direction_serialization(
    binding_args: &BindingArguments,
    fields: &[Field],
) -> TokenStream {
    if fields.iter().any(|f| match f.ty {
        FieldType::Direction => true,
        _ => false,
    }) {
        return quote!();
    }

    if let Some(direction) = binding_args.direction.as_ref() {
        quote!(map.serialize_entry("direction", #direction)?;)
    } else {
        macro_panic(
            Span::call_site(),
            "binding requires a 'direction' argument be specified",
        );
    }
}

pub fn binding_impl(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut definition: ItemStruct = parse(input)
        .map_err(|_| {
            macro_panic(
                Span::call_site(),
                "the 'binding' attribute can only be used on a struct",
            )
        })
        .unwrap();

    let binding_args =
        BindingArguments::from(match syn::parse_macro_input::parse::<AttributeArgs>(args) {
            Ok(args) => args,
            Err(e) => macro_panic(
                Span::call_site(),
                format!("failed to parse attribute arguments: {}", e),
            ),
        });

    let fields: Vec<Field> = definition.fields.iter().map(|f| f.into()).collect();
    drain_field_attributes(&mut definition.fields);

    let binding_name = &binding_args.name;
    let validate = binding_args.get_validation_call();
    let ident = &definition.ident;
    let default_direction = get_default_direction_serialization(&binding_args, &fields);
    let serializations = fields.iter().map(Field::get_serialization);
    let field_decls = fields.iter().filter_map(Field::get_field_decl);
    let field_matches = fields.iter().filter_map(Field::get_field_match);
    let required_checks = fields.iter().filter_map(Field::get_required_check);
    let field_assignments = fields.iter().map(Field::get_field_assignment);
    let quotable_decls = fields.iter().map(Field::get_quotable_decl);
    let quoteable_assignments = fields.iter().map(Field::get_quotable_assignment);

    quote!(
        #[derive(Debug, Clone)]
        #definition

        impl #ident {
            pub fn binding_type() -> &'static str {
                #binding_name
            }
        }

        impl serde::Serialize for #ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("type", #ident::binding_type())?;
                #default_direction
                #(#serializations)*
                map.end()
            }
        }

        impl From<(syn::AttributeArgs, proc_macro2::Span)> for #ident {
            fn from(__args_and_span: (syn::AttributeArgs, proc_macro2::Span)) -> Self {
                #(#field_decls)*

                crate::codegen::iter_attribute_args(&__args_and_span.0, |__key, __value| {
                    let __key_name = __key.to_string();

                    match __key_name.as_str() {
                        #(#field_matches,)*
                        _ => crate::codegen::macro_panic(__key.span(), format!("unsupported binding attribute argument '{}'", __key_name)),
                    };

                    true
                });

                #(#required_checks)*

                let __binding = #ident {
                    #(#field_assignments,)*
                };

                #validate

                __binding
            }
        }

        impl quote::ToTokens for #ident {
            fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
                #(#quotable_decls)*
                quote::quote!(
                    ::azure_functions::codegen::bindings::#ident {
                        #(#quoteable_assignments,)*
                    }
                ).to_tokens(tokens)
            }
        }
    )
    .into()
}
