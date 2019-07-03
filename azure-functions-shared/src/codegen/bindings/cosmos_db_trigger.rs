use azure_functions_shared_codegen::binding;
use std::borrow::Cow;

#[binding(name = "cosmosDBTrigger", direction = "in")]
pub struct CosmosDbTrigger {
    #[field(camel_case_value = true)]
    pub name: Cow<'static, str>,
    #[field(name = "connectionStringSetting")]
    pub connection: Cow<'static, str>,
    #[field(name = "databaseName")]
    pub database_name: Cow<'static, str>,
    #[field(name = "collectionName")]
    pub collection_name: Cow<'static, str>,
    #[field(name = "leaseConnectionStringSetting")]
    pub lease_connection: Option<Cow<'static, str>>,
    #[field(name = "leaseDatabaseName")]
    pub lease_database_name: Option<Cow<'static, str>>,
    #[field(name = "leaseCollectionName")]
    pub lease_collection_name: Option<Cow<'static, str>>,
    #[field(name = "createLeaseCollectionIfNotExists")]
    pub create_lease_collection: Option<bool>,
    #[field(name = "leasesCollectionThroughput")]
    pub lease_collection_throughput: Option<i64>,
    #[field(name = "leaseCollectionPrefix")]
    pub lease_collection_prefix: Option<Cow<'static, str>>,
    #[field(name = "feedPollDelay")]
    pub feed_poll_delay: Option<i64>,
    #[field(name = "leaseAcquireInterval")]
    pub lease_acquire_interval: Option<i64>,
    #[field(name = "leaseExpirationInterval")]
    pub lease_expiration_interval: Option<i64>,
    #[field(name = "leaseRenewInterval")]
    pub lease_renew_interval: Option<i64>,
    #[field(name = "checkpointFrequency")]
    pub checkpoint_frequency: Option<i64>,
    #[field(name = "maxItemsPerInvocation")]
    pub max_items_per_invocation: Option<i64>,
    #[field(name = "startFromBeginning")]
    pub start_from_beginning: Option<bool>,
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
        let binding = CosmosDbTrigger {
            name: Cow::from("name"),
            connection: Cow::from("connection"),
            database_name: Cow::from("database"),
            collection_name: Cow::from("collection"),
            lease_connection: Some(Cow::from("lease-connection")),
            lease_collection_name: Some(Cow::from("lease-collection")),
            lease_database_name: Some(Cow::from("lease-database")),
            create_lease_collection: Some(true),
            lease_collection_throughput: Some(54321),
            lease_collection_prefix: Some(Cow::from("lease-prefix")),
            feed_poll_delay: Some(12345),
            lease_acquire_interval: Some(12121),
            lease_expiration_interval: Some(10101),
            lease_renew_interval: Some(11111),
            checkpoint_frequency: Some(0),
            max_items_per_invocation: Some(42),
            start_from_beginning: Some(false),
        };

        assert_eq!(
            to_string(&binding).unwrap(),
            r#"{"type":"cosmosDBTrigger","direction":"in","name":"name","connectionStringSetting":"connection","databaseName":"database","collectionName":"collection","leaseConnectionStringSetting":"lease-connection","leaseDatabaseName":"lease-database","leaseCollectionName":"lease-collection","createLeaseCollectionIfNotExists":true,"leasesCollectionThroughput":54321,"leaseCollectionPrefix":"lease-prefix","feedPollDelay":12345,"leaseAcquireInterval":12121,"leaseExpirationInterval":10101,"leaseRenewInterval":11111,"checkpointFrequency":0,"maxItemsPerInvocation":42,"startFromBeginning":false}"#
        );
    }

    #[test]
    fn it_parses_attribute_arguments() {
        let binding: CosmosDbTrigger = (
            vec![
                parse_str::<NestedMeta>(r#"name = "name""#).unwrap(),
                parse_str::<NestedMeta>(r#"connection = "connection""#).unwrap(),
                parse_str::<NestedMeta>(r#"database_name = "database""#).unwrap(),
                parse_str::<NestedMeta>(r#"collection_name = "collection""#).unwrap(),
                parse_str::<NestedMeta>(r#"lease_connection = "lease-connection""#).unwrap(),
                parse_str::<NestedMeta>(r#"lease_collection_name = "lease-collection""#).unwrap(),
                parse_str::<NestedMeta>(r#"create_lease_collection = true"#).unwrap(),
                parse_str::<NestedMeta>(r#"lease_collection_throughput = 54321"#).unwrap(),
                parse_str::<NestedMeta>(r#"lease_collection_prefix = "lease-prefix""#).unwrap(),
                parse_str::<NestedMeta>(r#"feed_poll_delay = 12345"#).unwrap(),
                parse_str::<NestedMeta>(r#"lease_acquire_interval = 12121"#).unwrap(),
                parse_str::<NestedMeta>(r#"lease_expiration_interval = 10101"#).unwrap(),
                parse_str::<NestedMeta>(r#"lease_renew_interval = 11111"#).unwrap(),
                parse_str::<NestedMeta>(r#"checkpoint_frequency = 0"#).unwrap(),
                parse_str::<NestedMeta>(r#"max_items_per_invocation = 42"#).unwrap(),
                parse_str::<NestedMeta>(r#"start_from_beginning = false"#).unwrap(),
            ],
            Span::call_site(),
        )
            .into();

        assert_eq!(binding.name.as_ref(), "name");
        assert_eq!(binding.connection.as_ref(), "connection");
        assert_eq!(binding.database_name.as_ref(), "database");
        assert_eq!(binding.collection_name.as_ref(), "collection");
        assert_eq!(
            binding.lease_connection.as_ref().unwrap().as_ref(),
            "lease-connection"
        );
        assert_eq!(
            binding.lease_collection_name.as_ref().unwrap().as_ref(),
            "lease-collection"
        );
        assert_eq!(binding.create_lease_collection.unwrap(), true);
        assert_eq!(binding.lease_collection_throughput.unwrap(), 54321);
        assert_eq!(
            binding.lease_collection_prefix.as_ref().unwrap().as_ref(),
            "lease-prefix"
        );
        assert_eq!(binding.feed_poll_delay.unwrap(), 12345);
        assert_eq!(binding.lease_acquire_interval.unwrap(), 12121);
        assert_eq!(binding.lease_expiration_interval.unwrap(), 10101);
        assert_eq!(binding.lease_renew_interval.unwrap(), 11111);
        assert_eq!(binding.checkpoint_frequency.unwrap(), 0);
        assert_eq!(binding.max_items_per_invocation.unwrap(), 42);
        assert_eq!(binding.start_from_beginning.unwrap(), false);
    }

    #[test]
    fn it_requires_the_name_attribute_argument() {
        should_panic(
            || {
                let _: CosmosDbTrigger = (vec![], Span::call_site()).into();
            },
            "the 'name' argument is required for this binding",
        );
    }

    #[test]
    fn it_requires_the_name_attribute_be_a_string() {
        should_panic(
            || {
                let _: CosmosDbTrigger = (
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
                let _: CosmosDbTrigger = (
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
                let _: CosmosDbTrigger = (
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
                let _: CosmosDbTrigger = (
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
                let _: CosmosDbTrigger = (
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
                let _: CosmosDbTrigger = (
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
                let _: CosmosDbTrigger = (
                    vec![parse_str::<NestedMeta>(r#"collection_name = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'collection_name' argument",
        );
    }

    #[test]
    fn it_requires_the_lease_connection_attribute_be_a_string() {
        should_panic(
            || {
                let _: CosmosDbTrigger = (
                    vec![parse_str::<NestedMeta>(r#"lease_connection = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'lease_connection' argument",
        );
    }

    #[test]
    fn it_requires_the_create_lease_collection_attribute_be_a_boolean() {
        should_panic(
            || {
                let _: CosmosDbTrigger = (
                    vec![parse_str::<NestedMeta>(r#"create_lease_collection = 12345"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal boolean value for the 'create_lease_collection' argument",
        );
    }

    #[test]
    fn it_requires_the_lease_collection_throughput_attribute_be_an_integer() {
        should_panic(
            || {
                let _: CosmosDbTrigger = (
                    vec![
                        parse_str::<NestedMeta>(r#"lease_collection_throughput = "12345""#)
                            .unwrap(),
                    ],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal integer value for the 'lease_collection_throughput' argument",
        );
    }

    #[test]
    fn it_requires_the_lease_collection_prefix_attribute_be_a_string() {
        should_panic(
            || {
                let _: CosmosDbTrigger = (
                    vec![parse_str::<NestedMeta>(r#"lease_collection_prefix = false"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal string value for the 'lease_collection_prefix' argument",
        );
    }

    #[test]
    fn it_requires_the_feed_poll_delay_attribute_be_an_integer() {
        should_panic(
            || {
                let _: CosmosDbTrigger = (
                    vec![parse_str::<NestedMeta>(r#"feed_poll_delay = "12345""#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal integer value for the 'feed_poll_delay' argument",
        );
    }

    #[test]
    fn it_requires_the_lease_acquire_interval_attribute_be_an_integer() {
        should_panic(
            || {
                let _: CosmosDbTrigger = (
                    vec![parse_str::<NestedMeta>(r#"lease_acquire_interval = "12345""#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal integer value for the 'lease_acquire_interval' argument",
        );
    }

    #[test]
    fn it_requires_the_lease_expiration_interval_attribute_be_an_integer() {
        should_panic(
            || {
                let _: CosmosDbTrigger = (
                    vec![
                        parse_str::<NestedMeta>(r#"lease_expiration_interval = "12345""#).unwrap(),
                    ],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal integer value for the 'lease_expiration_interval' argument",
        );
    }

    #[test]
    fn it_requires_the_lease_renew_interval_attribute_be_an_integer() {
        should_panic(
            || {
                let _: CosmosDbTrigger = (
                    vec![parse_str::<NestedMeta>(r#"lease_renew_interval = "12345""#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal integer value for the 'lease_renew_interval' argument",
        );
    }

    #[test]
    fn it_requires_the_checkpoint_frequency_attribute_be_an_integer() {
        should_panic(
            || {
                let _: CosmosDbTrigger = (
                    vec![parse_str::<NestedMeta>(r#"checkpoint_frequency = "12345""#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal integer value for the 'checkpoint_frequency' argument",
        );
    }

    #[test]
    fn it_requires_the_max_items_per_invocation_attribute_be_an_integer() {
        should_panic(
            || {
                let _: CosmosDbTrigger = (
                    vec![parse_str::<NestedMeta>(r#"max_items_per_invocation = "12345""#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal integer value for the 'max_items_per_invocation' argument",
        );
    }

    #[test]
    fn it_requires_the_start_from_beginning_attribute_be_a_boolean() {
        should_panic(
            || {
                let _: CosmosDbTrigger = (
                    vec![parse_str::<NestedMeta>(r#"start_from_beginning = 12345"#).unwrap()],
                    Span::call_site(),
                )
                    .into();
            },
            "expected a literal boolean value for the 'start_from_beginning' argument",
        );
    }

    #[test]
    fn it_converts_to_tokens() {
        let binding = CosmosDbTrigger {
            name: Cow::from("name"),
            connection: Cow::from("connection"),
            database_name: Cow::from("database"),
            collection_name: Cow::from("collection"),
            lease_connection: Some(Cow::from("lease-connection")),
            lease_collection_name: Some(Cow::from("lease-collection")),
            lease_database_name: Some(Cow::from("lease-database")),
            create_lease_collection: Some(true),
            lease_collection_throughput: Some(54321),
            lease_collection_prefix: Some(Cow::from("lease-prefix")),
            feed_poll_delay: Some(12345),
            lease_acquire_interval: Some(12121),
            lease_expiration_interval: Some(10101),
            lease_renew_interval: Some(11111),
            checkpoint_frequency: Some(0),
            max_items_per_invocation: Some(42),
            start_from_beginning: Some(false),
        };

        let mut stream = TokenStream::new();
        binding.to_tokens(&mut stream);
        let mut tokens = stream.to_string();
        tokens.retain(|c| c != ' ');

        assert_eq!(tokens, r#"::azure_functions::codegen::bindings::CosmosDbTrigger{name:::std::borrow::Cow::Borrowed("name"),connection:::std::borrow::Cow::Borrowed("connection"),database_name:::std::borrow::Cow::Borrowed("database"),collection_name:::std::borrow::Cow::Borrowed("collection"),lease_connection:Some(::std::borrow::Cow::Borrowed("lease-connection")),lease_database_name:Some(::std::borrow::Cow::Borrowed("lease-database")),lease_collection_name:Some(::std::borrow::Cow::Borrowed("lease-collection")),create_lease_collection:Some(true),lease_collection_throughput:Some(54321i64),lease_collection_prefix:Some(::std::borrow::Cow::Borrowed("lease-prefix")),feed_poll_delay:Some(12345i64),lease_acquire_interval:Some(12121i64),lease_expiration_interval:Some(10101i64),lease_renew_interval:Some(11111i64),checkpoint_frequency:Some(0i64),max_items_per_invocation:Some(42i64),start_from_beginning:Some(false),}"#);
    }
}
