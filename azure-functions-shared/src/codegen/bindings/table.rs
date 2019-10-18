use crate::codegen::bindings::Direction;
use azure_functions_shared_codegen::binding;
use std::borrow::Cow;

#[binding(name = "table")]
pub struct Table {
    pub direction: Direction,
    #[field(camel_case_value = true)]
    pub name: Cow<'static, str>,
    #[field(name = "tableName")]
    pub table_name: Cow<'static, str>,
    #[field(name = "partitionKey")]
    pub partition_key: Option<Cow<'static, str>>,
    #[field(name = "rowKey")]
    pub row_key: Option<Cow<'static, str>>,
    pub filter: Option<Cow<'static, str>>,
    pub take: Option<i64>,
    pub connection: Option<Cow<'static, str>>,
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
        let binding = Table {
            direction: Direction::In,
            name: Cow::from("foo"),
            table_name: Cow::from("bar"),
            partition_key: Some(Cow::from("baz")),
            row_key: Some(Cow::from("cake")),
            filter: Some(Cow::from("is")),
            take: Some(10),
            connection: Some(Cow::from("a lie")),
        };

        assert_eq!(
            to_string(&binding).unwrap(),
            r#"{"type":"table","direction":"in","name":"foo","tableName":"bar","partitionKey":"baz","rowKey":"cake","filter":"is","take":10,"connection":"a lie"}"#
        );
    }

    #[test]
    fn it_parses_attribute_arguments() {
        let binding: Table = (
            vec![
                parse_str::<NestedMeta>(r#"name = "foo""#).unwrap(),
                parse_str::<NestedMeta>(r#"table_name = "bar""#).unwrap(),
                parse_str::<NestedMeta>(r#"partition_key = "baz""#).unwrap(),
                parse_str::<NestedMeta>(r#"row_key = "cake""#).unwrap(),
                parse_str::<NestedMeta>(r#"filter = "is""#).unwrap(),
                parse_str::<NestedMeta>(r#"take = 42"#).unwrap(),
                parse_str::<NestedMeta>(r#"connection = "a lie""#).unwrap(),
            ],
            Span::call_site(),
        )
            .into();

        assert_eq!(binding.direction, Direction::In);
        assert_eq!(binding.name.as_ref(), "foo");
        assert_eq!(binding.table_name.as_ref(), "bar");
        assert_eq!(binding.partition_key.unwrap().as_ref(), "baz");
        assert_eq!(binding.row_key.unwrap().as_ref(), "cake");
        assert_eq!(binding.filter.unwrap().as_ref(), "is");
        assert_eq!(binding.take.unwrap(), 42);
        assert_eq!(binding.connection.unwrap().as_ref(), "a lie");
    }

    #[test]
    fn it_requires_the_name_attribute_argument() {
        should_panic(
            || {
                let _: Table = (vec![], Span::call_site()).into();
            },
            "the 'name' argument is required for this binding",
        );
    }

    #[test]
    fn it_requires_the_name_attribute_be_a_string() {
        should_panic(
            || {
                let _: Table = (
                    vec![parse_str::<NestedMeta>(r#"name = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'name' argument",
        );
    }

    #[test]
    fn it_requires_the_table_name_attribute_argument() {
        should_panic(
            || {
                let _: Table = (
                    vec![parse_str::<NestedMeta>(r#"name = "foo""#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "the 'table_name' argument is required for this binding",
        );
    }

    #[test]
    fn it_requires_the_table_name_attribute_be_a_string() {
        should_panic(
            || {
                let _: Table = (
                    vec![parse_str::<NestedMeta>(r#"table_name = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'table_name' argument",
        );
    }

    #[test]
    fn it_requires_the_partition_key_attribute_be_a_string() {
        should_panic(
            || {
                let _: Table = (
                    vec![parse_str::<NestedMeta>(r#"partition_key = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'partition_key' argument",
        );
    }

    #[test]
    fn it_requires_the_row_key_attribute_be_a_string() {
        should_panic(
            || {
                let _: Table = (
                    vec![parse_str::<NestedMeta>(r#"row_key = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'row_key' argument",
        );
    }

    #[test]
    fn it_requires_the_filter_be_a_string() {
        should_panic(
            || {
                let _: Table = (
                    vec![parse_str::<NestedMeta>(r#"filter = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'filter' argument",
        );
    }

    #[test]
    fn it_requires_the_take_attribute_be_a_string() {
        should_panic(
            || {
                let _: Table = (
                    vec![parse_str::<NestedMeta>(r#"take = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal integer value for the 'take' argument",
        );
    }

    #[test]
    fn it_requires_the_connection_attribute_be_a_string() {
        should_panic(
            || {
                let _: Table = (
                    vec![parse_str::<NestedMeta>(r#"connection = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'connection' argument",
        );
    }

    #[test]
    fn it_converts_to_tokens() {
        let binding = Table {
            direction: Direction::In,
            name: Cow::from("foo"),
            table_name: Cow::from("bar"),
            partition_key: Some(Cow::from("baz")),
            row_key: Some(Cow::from("cake")),
            filter: Some(Cow::from("is")),
            take: Some(10),
            connection: Some(Cow::from("a lie")),
        };

        let mut stream = TokenStream::new();
        binding.to_tokens(&mut stream);
        let mut tokens = stream.to_string();
        tokens.retain(|c| c != ' ');

        assert_eq!(
            tokens,
            r#"::azure_functions::codegen::bindings::Table{direction:::azure_functions::codegen::bindings::Direction::In,name:::std::borrow::Cow::Borrowed("foo"),table_name:::std::borrow::Cow::Borrowed("bar"),partition_key:Some(::std::borrow::Cow::Borrowed("baz")),row_key:Some(::std::borrow::Cow::Borrowed("cake")),filter:Some(::std::borrow::Cow::Borrowed("is")),take:Some(10i64),connection:Some(::std::borrow::Cow::Borrowed("alie")),}"#
        );
    }
}
