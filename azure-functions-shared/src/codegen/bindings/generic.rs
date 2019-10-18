use crate::codegen::{
    bindings::Direction,
    get_string_value, iter_attribute_args, macro_panic,
    quotable::{QuotableBorrowedStr, QuotableDirection},
    Value,
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use serde::{ser::SerializeMap, Serialize, Serializer};
use std::borrow::Cow;
use syn::{spanned::Spanned, AttributeArgs, Lit};

#[derive(Debug, Clone)]
pub struct Generic {
    pub ty: Cow<'static, str>,
    pub direction: Direction,
    pub name: Cow<'static, str>,
    pub values: Cow<'static, [(Cow<'static, str>, Value)]>,
}

impl Generic {
    pub fn binding_type(&self) -> &str {
        self.ty.as_ref()
    }
}

impl Serialize for Generic {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", &self.ty)?;
        map.serialize_entry("direction", &self.direction)?;
        map.serialize_entry("name", &self.name)?;

        for value in self.values.iter() {
            map.serialize_entry(&value.0, &value.1)?;
        }

        map.end()
    }
}

impl From<(AttributeArgs, Span)> for Generic {
    fn from(args_and_span: (AttributeArgs, Span)) -> Self {
        let mut ty = None;
        let mut name = None;
        let mut values = Vec::new();

        iter_attribute_args(&args_and_span.0, |key, value| {
            let key_name = key.to_string();

            match key_name.as_str() {
                "type" => {
                    let binding_type = get_string_value("type", value);
                    if binding_type.to_lowercase() == "httptrigger" {
                        macro_panic(
                            value.span(),
                            "using a generic binding of type 'httpTrigger' is not supported",
                        );
                    }
                    ty = Some(binding_type);
                }
                "name" => name = Some(get_string_value("name", value)),
                _ => {
                    match value {
                        Lit::Str(s) => {
                            values.push((Cow::from(key_name), Value::String(Cow::from(s.value()))));
                        }
                        Lit::Int(i) => {
                            values.push((
                                Cow::from(key_name),
                                Value::Integer(i.base10_parse::<i64>().unwrap()),
                            ));
                        }
                        Lit::Bool(b) => values.push((Cow::from(key_name), Value::Boolean(b.value))),
                        _ => macro_panic(value.span(), "expected a string, integer, or boolean"),
                    };
                }
            };

            true
        });

        if ty.is_none() {
            macro_panic(
                args_and_span.1,
                "the 'type' argument is required for this binding",
            );
        }

        if name.is_none() {
            macro_panic(
                args_and_span.1,
                "the 'name' argument is required for this binding",
            );
        }

        Generic {
            ty: Cow::from(ty.unwrap()),
            direction: Direction::In,
            name: Cow::from(name.unwrap()),
            values: Cow::from(values),
        }
    }
}

impl ToTokens for Generic {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ty = QuotableBorrowedStr(&self.ty);
        let direction = QuotableDirection(self.direction);
        let name = QuotableBorrowedStr(&self.name);
        let values = self.values.iter().map(|v| {
            let name = QuotableBorrowedStr(&v.0);
            let value = &v.1;
            quote!((#name, #value))
        });

        quote!(
            ::azure_functions::codegen::bindings::Generic {
                ty: #ty,
                direction: #direction,
                name: #name,
                values: ::std::borrow::Cow::Borrowed(&[#(#values,)*]),
            }
        )
        .to_tokens(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::tests::should_panic;
    use proc_macro2::{Span, TokenStream};
    use quote::ToTokens;
    use serde_json::to_string;
    use syn::{parse_str, NestedMeta};

    #[test]
    fn it_serializes_to_json() {
        let binding = Generic {
            ty: Cow::from("test"),
            direction: Direction::InOut,
            name: Cow::from("foo"),
            values: Cow::from(vec![
                (Cow::from("bar"), Value::String(Cow::from("hello"))),
                (Cow::from("baz"), Value::Integer(42)),
                (Cow::from("jam"), Value::Boolean(true)),
            ]),
        };

        assert_eq!(
            to_string(&binding).unwrap(),
            r#"{"type":"test","direction":"inout","name":"foo","bar":"hello","baz":42,"jam":true}"#
        );
    }

    #[test]
    fn it_parses_attribute_arguments() {
        let binding: Generic = (
            vec![
                parse_str::<NestedMeta>(r#"type = "test""#).unwrap(),
                parse_str::<NestedMeta>(r#"name = "foo""#).unwrap(),
                parse_str::<NestedMeta>(r#"bar = "hello""#).unwrap(),
                parse_str::<NestedMeta>(r#"baz = 42"#).unwrap(),
                parse_str::<NestedMeta>(r#"jam = true"#).unwrap(),
            ],
            Span::call_site(),
        )
            .into();

        assert_eq!(binding.ty.as_ref(), "test");
        assert_eq!(binding.direction, Direction::In);
        assert_eq!(binding.name.as_ref(), "foo");
        assert_eq!(
            binding.values.as_ref(),
            [
                (Cow::from("bar"), Value::String(Cow::from("hello"))),
                (Cow::from("baz"), Value::Integer(42)),
                (Cow::from("jam"), Value::Boolean(true)),
            ]
        );
    }

    #[test]
    fn it_requires_the_type_attribute_argument() {
        should_panic(
            || {
                let _: Generic = (vec![], Span::call_site()).into();
            },
            "the 'type' argument is required for this binding",
        );
    }

    #[test]
    fn it_requires_the_type_attribute_be_a_string() {
        should_panic(
            || {
                let _: Generic = (
                    vec![parse_str::<NestedMeta>(r#"type = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'type' argument",
        );
    }

    #[test]
    fn it_requires_the_name_attribute_argument() {
        should_panic(
            || {
                let _: Generic = (
                    vec![parse_str::<NestedMeta>(r#"type = "foo""#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "the 'name' argument is required for this binding",
        );
    }

    #[test]
    fn it_requires_the_name_attribute_be_a_string() {
        should_panic(
            || {
                let _: Generic = (
                    vec![parse_str::<NestedMeta>(r#"name = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'name' argument",
        );
    }

    #[test]
    fn it_converts_to_tokens() {
        let binding = Generic {
            ty: Cow::from("test"),
            direction: Direction::InOut,
            name: Cow::from("foo"),
            values: Cow::from(vec![
                (Cow::from("bar"), Value::String(Cow::from("hello"))),
                (Cow::from("baz"), Value::Integer(42)),
                (Cow::from("jam"), Value::Boolean(true)),
            ]),
        };

        let mut stream = TokenStream::new();
        binding.to_tokens(&mut stream);
        let mut tokens = stream.to_string();
        tokens.retain(|c| c != ' ');

        assert_eq!(tokens, r#"::azure_functions::codegen::bindings::Generic{ty:::std::borrow::Cow::Borrowed("test"),direction:::azure_functions::codegen::bindings::Direction::InOut,name:::std::borrow::Cow::Borrowed("foo"),values:::std::borrow::Cow::Borrowed(&[(::std::borrow::Cow::Borrowed("bar"),::azure_functions::codegen::Value::String(::std::borrow::Cow::Borrowed("hello"))),(::std::borrow::Cow::Borrowed("baz"),::azure_functions::codegen::Value::Integer(42i64)),(::std::borrow::Cow::Borrowed("jam"),::azure_functions::codegen::Value::Boolean(true)),]),}"#);
    }
}
