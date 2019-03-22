use crate::codegen::bindings::Direction;
use azure_functions_shared_codegen::binding;
use std::borrow::Cow;

#[binding(name = "cosmosDB")]
pub struct CosmosDb {
    pub direction: Direction,
    #[field(camel_case_value = true)]
    pub name: Cow<'static, str>,
    #[field(name = "connectionStringSetting")]
    pub connection: Cow<'static, str>,
    #[field(name = "databaseName")]
    pub database_name: Cow<'static, str>,
    #[field(name = "collectionName")]
    pub collection_name: Cow<'static, str>,
    #[field(name = "partitionKey")]
    pub partition_key: Option<Cow<'static, str>>,

    // OUTPUT ONLY
    #[field(name = "createIfNotExists")]
    pub create_collection: Option<bool>,
    #[field(name = "collectionThroughput")]
    pub collection_throughput: Option<i64>,

    // INPUT ONLY
    #[field(name = "id")]
    pub id: Option<Cow<'static, str>>,
    #[field(name = "sqlQuery")]
    pub sql_query: Option<Cow<'static, str>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::bindings::tests::should_panic;
    use proc_macro2::{Span, TokenStream};
    use quote::ToTokens;
    use serde_json::to_string;
    use syn::{parse_str, NestedMeta};

    #[test]
    fn it_serializes_to_json() {
        let binding = CosmosDb {
            direction: Direction::In,
            name: Cow::from("name"),
            connection: Cow::from("connection"),
            database_name: Cow::from("database"),
            collection_name: Cow::from("collection"),
            partition_key: Some(Cow::from("partition")),
            create_collection: Some(true),
            collection_throughput: Some(12345),
            id: Some(Cow::from("id")),
            sql_query: Some(Cow::from("query")),
        };

        assert_eq!(
            to_string(&binding).unwrap(),
            r#"{"type":"cosmosDB","direction":"in","name":"name","connectionStringSetting":"connection","databaseName":"database","collectionName":"collection","partitionKey":"partition","createIfNotExists":true,"collectionThroughput":12345,"id":"id","sqlQuery":"query"}"#
        );
    }

    #[test]
    fn it_parses_attribute_arguments() {
        let binding: CosmosDb = (
            vec![
                parse_str::<NestedMeta>(r#"name = "name""#).unwrap(),
                parse_str::<NestedMeta>(r#"connection = "connection""#).unwrap(),
                parse_str::<NestedMeta>(r#"database_name = "database""#).unwrap(),
                parse_str::<NestedMeta>(r#"collection_name = "collection""#).unwrap(),
                parse_str::<NestedMeta>(r#"partition_key = "partition""#).unwrap(),
                parse_str::<NestedMeta>(r#"create_collection = true"#).unwrap(),
                parse_str::<NestedMeta>(r#"collection_throughput = 12345"#).unwrap(),
                parse_str::<NestedMeta>(r#"id = "id""#).unwrap(),
                parse_str::<NestedMeta>(r#"sql_query = "query""#).unwrap(),
            ],
            Span::call_site(),
        )
            .into();

        assert_eq!(binding.name.as_ref(), "name");
        assert_eq!(binding.connection.as_ref(), "connection");
        assert_eq!(binding.database_name.as_ref(), "database");
        assert_eq!(binding.collection_name.as_ref(), "collection");
        assert_eq!(
            binding.partition_key.as_ref().unwrap().as_ref(),
            "partition"
        );
        assert_eq!(binding.create_collection.unwrap(), true);
        assert_eq!(binding.collection_throughput.unwrap(), 12345);
        assert_eq!(binding.id.as_ref().unwrap().as_ref(), "id");
        assert_eq!(binding.sql_query.as_ref().unwrap().as_ref(), "query");
    }

    #[test]
    fn it_requires_the_name_attribute_argument() {
        should_panic(
            || {
                let _: CosmosDb = (vec![], Span::call_site()).into();
            },
            "the 'name' argument is required for this binding",
        );
    }

    #[test]
    fn it_requires_the_name_attribute_be_a_string() {
        should_panic(
            || {
                let _: CosmosDb = (
                    vec![parse_str::<NestedMeta>(r#"name = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'name' argument",
        );
    }

    #[test]
    fn it_requires_the_connection_attribute_argument() {
        should_panic(
            || {
                let _: CosmosDb = (
                    vec![parse_str::<NestedMeta>(r#"name = "name""#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "the 'connection' argument is required for this binding",
        );
    }

    #[test]
    fn it_requires_the_connection_attribute_be_a_string() {
        should_panic(
            || {
                let _: CosmosDb = (
                    vec![parse_str::<NestedMeta>(r#"connection = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'connection' argument",
        );
    }

    #[test]
    fn it_requires_the_database_name_attribute_argument() {
        should_panic(
            || {
                let _: CosmosDb = (
                    vec![
                        parse_str::<NestedMeta>(r#"name = "name""#).unwrap(),
                        parse_str::<NestedMeta>(r#"connection = "connection""#).unwrap(),
                    ],
                    Span::call_site(),
                )
                    .into();
            },
            "the 'database_name' argument is required for this binding",
        );
    }

    #[test]
    fn it_requires_the_database_name_attribute_be_a_string() {
        should_panic(
            || {
                let _: CosmosDb = (
                    vec![parse_str::<NestedMeta>(r#"database_name = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'database_name' argument",
        );
    }

    #[test]
    fn it_requires_the_collection_name_attribute_argument() {
        should_panic(
            || {
                let _: CosmosDb = (
                    vec![
                        parse_str::<NestedMeta>(r#"name = "name""#).unwrap(),
                        parse_str::<NestedMeta>(r#"connection = "connection""#).unwrap(),
                        parse_str::<NestedMeta>(r#"database_name = "database""#).unwrap(),
                    ],
                    Span::call_site(),
                )
                    .into();
            },
            "the 'collection_name' argument is required for this binding",
        );
    }

    #[test]
    fn it_requires_the_collection_name_attribute_be_a_string() {
        should_panic(
            || {
                let _: CosmosDb = (
                    vec![parse_str::<NestedMeta>(r#"collection_name = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'collection_name' argument",
        );
    }

    #[test]
    fn it_requires_the_partition_key_attribute_be_a_string() {
        should_panic(
            || {
                let _: CosmosDb = (
                    vec![parse_str::<NestedMeta>(r#"partition_key = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'partition_key' argument",
        );
    }

    #[test]
    fn it_requires_the_create_collection_attribute_be_a_bool() {
        should_panic(
            || {
                let _: CosmosDb = (
                    vec![parse_str::<NestedMeta>(r#"create_collection = 1"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal boolean value for the 'create_collection' argument",
        );
    }

    #[test]
    fn it_requires_the_collection_throughput_attribute_be_an_integer() {
        should_panic(
            || {
                let _: CosmosDb = (
                    vec![parse_str::<NestedMeta>(r#"collection_throughput = "wrong""#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal integer value for the 'collection_throughput' argument",
        );
    }

    #[test]
    fn it_requires_the_id_attribute_be_a_string() {
        should_panic(
            || {
                let _: CosmosDb = (
                    vec![parse_str::<NestedMeta>(r#"id = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'id' argument",
        );
    }

    #[test]
    fn it_requires_the_sql_query_attribute_be_a_string() {
        should_panic(
            || {
                let _: CosmosDb = (
                    vec![parse_str::<NestedMeta>(r#"sql_query = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'sql_query' argument",
        );
    }

    #[test]
    fn it_converts_to_tokens() {
        let binding = CosmosDb {
            direction: Direction::In,
            name: Cow::from("name"),
            connection: Cow::from("connection"),
            database_name: Cow::from("database"),
            collection_name: Cow::from("collection"),
            partition_key: Some(Cow::from("partition")),
            create_collection: Some(true),
            collection_throughput: Some(12345),
            id: Some(Cow::from("id")),
            sql_query: Some(Cow::from("query")),
        };

        let mut stream = TokenStream::new();
        binding.to_tokens(&mut stream);
        let mut tokens = stream.to_string();
        tokens.retain(|c| c != ' ');

        assert_eq!(tokens, r#"::azure_functions::codegen::bindings::CosmosDb{direction:::azure_functions::codegen::bindings::Direction::In,name:::std::borrow::Cow::Borrowed("name"),connection:::std::borrow::Cow::Borrowed("connection"),database_name:::std::borrow::Cow::Borrowed("database"),collection_name:::std::borrow::Cow::Borrowed("collection"),partition_key:Some(::std::borrow::Cow::Borrowed("partition")),create_collection:Some(true),collection_throughput:Some(12345i64),id:Some(::std::borrow::Cow::Borrowed("id")),sql_query:Some(::std::borrow::Cow::Borrowed("query")),}"#);
    }
}
