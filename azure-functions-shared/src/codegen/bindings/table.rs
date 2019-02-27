use crate::codegen::{
    bindings::Direction,
    quotable::{QuotableBorrowedStr, QuotableDirection, QuotableOption},
    AttributeArguments, TryFrom,
};
use crate::util::to_camel_case;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use serde::{ser::SerializeMap, Serialize, Serializer};
use std::borrow::Cow;
use syn::{spanned::Spanned, Lit};

pub const TABLE_TYPE: &str = "table";

#[derive(Debug, Clone)]
pub struct Table {
    pub name: Cow<'static, str>,
    pub direction: Direction,
    pub table_name: Cow<'static, str>,
    pub partition_key: Option<Cow<'static, str>>,
    pub row_key: Option<Cow<'static, str>>,
    pub filter: Option<Cow<'static, str>>,
    pub take: Option<u64>,
    pub connection: Option<Cow<'static, str>>,
}

// TODO: when https://github.com/serde-rs/serde/issues/760 is resolved, remove implementation in favor of custom Serialize derive
// The fix would allow us to set the constant `type` entry rather than having to emit it manually.
impl Serialize for Table {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("type", TABLE_TYPE)?;
        map.serialize_entry("direction", &self.direction)?;
        map.serialize_entry("tableName", &self.table_name)?;

        if let Some(partition_key) = self.partition_key.as_ref() {
            map.serialize_entry("partitionKey", partition_key)?;
        }

        if let Some(row_key) = self.row_key.as_ref() {
            map.serialize_entry("rowKey", row_key)?;
        }

        if let Some(filter) = self.filter.as_ref() {
            map.serialize_entry("filter", filter)?;
        }

        if let Some(take) = self.take.as_ref() {
            map.serialize_entry("take", take)?;
        }

        if let Some(connection) = self.connection.as_ref() {
            map.serialize_entry("connection", connection)?;
        }

        map.end()
    }
}

impl TryFrom<AttributeArguments> for Table {
    type Error = (Span, String);

    fn try_from(args: AttributeArguments) -> Result<Self, Self::Error> {
        let mut name = None;
        let mut table_name = None;
        let mut partition_key = None;
        let mut row_key = None;
        let mut filter = None;
        let mut take = None;
        let mut connection = None;

        for (key, value) in args.list.iter() {
            let key_str = key.to_string();

            match key_str.as_str() {
                "name" => match value {
                    Lit::Str(s) => {
                        name = Some(Cow::Owned(to_camel_case(&s.value())));
                    }
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal string value for the 'name' argument".to_string(),
                        ));
                    }
                },
                "table_name" => match value {
                    Lit::Str(s) => {
                        table_name = Some(Cow::Owned(s.value()));
                    }
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal string value for the 'table_name' argument"
                                .to_string(),
                        ));
                    }
                },
                "partition_key" => match value {
                    Lit::Str(s) => {
                        partition_key = Some(Cow::Owned(s.value()));
                    }
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal string value for the 'partition_key' argument"
                                .to_string(),
                        ));
                    }
                },
                "row_key" => match value {
                    Lit::Str(s) => {
                        row_key = Some(Cow::Owned(s.value()));
                    }
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal string value for the 'row_key' argument"
                                .to_string(),
                        ));
                    }
                },
                "filter" => match value {
                    Lit::Str(s) => {
                        filter = Some(Cow::Owned(s.value()));
                    }
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal string value for the 'filter' argument".to_string(),
                        ));
                    }
                },
                "take" => match value {
                    Lit::Int(i) => {
                        take = Some(i.value());
                    }
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal integer value for the 'take' argument".to_string(),
                        ));
                    }
                },
                "connection" => match value {
                    Lit::Str(s) => {
                        connection = Some(Cow::Owned(s.value()));
                    }
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal string value for the 'connection' argument"
                                .to_string(),
                        ));
                    }
                },
                _ => {
                    return Err((
                        key.span(),
                        format!("unsupported binding attribute argument '{}'", key_str),
                    ));
                }
            };
        }

        if table_name.is_none() {
            return Err((
                args.span,
                "the 'table_name' argument is required for table bindings.".to_string(),
            ));
        }

        Ok(Table {
            name: name.unwrap(),
            table_name: table_name.unwrap(),
            partition_key,
            row_key,
            filter,
            take,
            connection,
            direction: Direction::In,
        })
    }
}

impl ToTokens for Table {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = QuotableBorrowedStr(&self.name);
        let table_name = QuotableBorrowedStr(&self.table_name);
        let partition_key =
            QuotableOption(self.partition_key.as_ref().map(|x| QuotableBorrowedStr(x)));
        let row_key = QuotableOption(self.row_key.as_ref().map(|x| QuotableBorrowedStr(x)));
        let filter = QuotableOption(self.filter.as_ref().map(|x| QuotableBorrowedStr(x)));
        let take = QuotableOption(self.take.as_ref());
        let connection = QuotableOption(self.connection.as_ref().map(|x| QuotableBorrowedStr(x)));
        let direction = QuotableDirection(self.direction.clone());

        quote!(::azure_functions::codegen::bindings::Table {
            name: #name,
            table_name: #table_name,
            partition_key: #partition_key,
            row_key: #row_key,
            filter: #filter,
            take: #take,
            connection: #connection,
            direction: #direction,
        })
        .to_tokens(tokens)
    }
}
