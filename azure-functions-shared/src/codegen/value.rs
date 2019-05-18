use crate::codegen::quotable::QuotableBorrowedStr;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use serde::{Serialize, Serializer};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(Cow<'static, str>),
    Integer(i64),
    Boolean(bool),
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::String(s) => serializer.serialize_str(s),
            Value::Integer(i) => serializer.serialize_i64(*i),
            Value::Boolean(b) => serializer.serialize_bool(*b),
        }
    }
}

impl ToTokens for Value {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Value::String(s) => {
                let s = QuotableBorrowedStr(s);
                quote!(::azure_functions::codegen::Value::String(#s))
            }
            Value::Integer(i) => quote!(::azure_functions::codegen::Value::Integer(#i)),
            Value::Boolean(b) => quote!(::azure_functions::codegen::Value::Boolean(#b)),
        }
        .to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;

    #[test]
    fn it_serializes_string_to_json() {
        let value = Value::String(Cow::from("foo"));

        assert_eq!(to_string(&value).unwrap(), r#""foo""#);
    }

    #[test]
    fn it_serializes_integer_to_json() {
        let value = Value::Integer(42);

        assert_eq!(to_string(&value).unwrap(), "42");
    }

    #[test]
    fn it_serializes_boolean_to_json() {
        let value = Value::Boolean(true);

        assert_eq!(to_string(&value).unwrap(), "true");

        let value = Value::Boolean(false);

        assert_eq!(to_string(&value).unwrap(), "false");
    }

    #[test]
    fn it_converts_string_to_tokens() {
        let value = Value::String(Cow::from("foo"));

        let mut stream = TokenStream::new();
        value.to_tokens(&mut stream);
        let mut tokens = stream.to_string();
        tokens.retain(|c| c != ' ');

        assert_eq!(
            tokens,
            r#"::azure_functions::codegen::Value::String(::std::borrow::Cow::Borrowed("foo"))"#
        );
    }

    #[test]
    fn it_converts_integer_to_tokens() {
        let value = Value::Integer(42);

        let mut stream = TokenStream::new();
        value.to_tokens(&mut stream);
        let mut tokens = stream.to_string();
        tokens.retain(|c| c != ' ');

        assert_eq!(
            tokens,
            r#"::azure_functions::codegen::Value::Integer(42i64)"#
        );
    }

    #[test]
    fn it_converts_boolean_to_tokens() {
        let value = Value::Boolean(true);

        let mut stream = TokenStream::new();
        value.to_tokens(&mut stream);
        let mut tokens = stream.to_string();
        tokens.retain(|c| c != ' ');

        assert_eq!(
            tokens,
            r#"::azure_functions::codegen::Value::Boolean(true)"#
        );

        let value = Value::Boolean(false);

        let mut stream = TokenStream::new();
        value.to_tokens(&mut stream);
        let mut tokens = stream.to_string();
        tokens.retain(|c| c != ' ');

        assert_eq!(
            tokens,
            r#"::azure_functions::codegen::Value::Boolean(false)"#
        );
    }
}
