use crate::util::{
    to_camel_case, AttributeArguments, QuotableBorrowedStr, QuotableDirection, QuotableOption,
};
use azure_functions_shared::codegen;
use proc_macro::Diagnostic;
use quote::ToTokens;
use std::borrow::Cow;
use std::convert::TryFrom;
use syn::spanned::Spanned;
use syn::Lit;

pub struct Table<'a>(pub Cow<'a, codegen::bindings::Table>);

impl TryFrom<AttributeArguments> for Table<'_> {
    type Error = Diagnostic;

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
                        return Err(value
                            .span()
                            .unstable()
                            .error("expected a literal string value for the 'name' argument"));
                    }
                },
                "table_name" => match value {
                    Lit::Str(s) => {
                        table_name = Some(Cow::Owned(s.value()));
                    }
                    _ => {
                        return Err(value.span().unstable().error(
                            "expected a literal string value for the 'table_name' argument",
                        ));
                    }
                },
                "partition_key" => match value {
                    Lit::Str(s) => {
                        partition_key = Some(Cow::Owned(s.value()));
                    }
                    _ => {
                        return Err(value.span().unstable().error(
                            "expected a literal string value for the 'partition_key' argument",
                        ));
                    }
                },
                "row_key" => match value {
                    Lit::Str(s) => {
                        row_key = Some(Cow::Owned(s.value()));
                    }
                    _ => {
                        return Err(value
                            .span()
                            .unstable()
                            .error("expected a literal string value for the 'row_key' argument"));
                    }
                },
                "filter" => match value {
                    Lit::Str(s) => {
                        filter = Some(Cow::Owned(s.value()));
                    }
                    _ => {
                        return Err(value
                            .span()
                            .unstable()
                            .error("expected a literal string value for the 'filter' argument"));
                    }
                },
                "take" => match value {
                    Lit::Int(i) => {
                        take = Some(i.value());
                    }
                    _ => {
                        return Err(value
                            .span()
                            .unstable()
                            .error("expected a literal integer value for the 'take' argument"));
                    }
                },
                "connection" => match value {
                    Lit::Str(s) => {
                        connection = Some(Cow::Owned(s.value()));
                    }
                    _ => {
                        return Err(value.span().unstable().error(
                            "expected a literal string value for the 'connection' argument",
                        ));
                    }
                },
                _ => {
                    return Err(key.span().unstable().error(format!(
                        "unsupported binding attribute argument '{}'",
                        key_str
                    )));
                }
            };
        }

        if table_name.is_none() {
            return Err(args
                .span
                .error("the 'table_name' argument is required for table bindings."));
        }

        Ok(Table(Cow::Owned(codegen::bindings::Table {
            name: name.expect("expected a name for a table binding"),
            table_name: table_name.expect("expected a table name for a table binding"),
            partition_key,
            row_key,
            filter,
            take,
            connection,
            direction: codegen::Direction::In,
        })))
    }
}

impl ToTokens for Table<'_> {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        let name = QuotableBorrowedStr(&self.0.name);
        let table_name = QuotableBorrowedStr(&self.0.table_name);
        let partition_key = QuotableOption(
            self.0
                .partition_key
                .as_ref()
                .map(|x| QuotableBorrowedStr(x)),
        );
        let row_key = QuotableOption(self.0.row_key.as_ref().map(|x| QuotableBorrowedStr(x)));
        let filter = QuotableOption(self.0.filter.as_ref().map(|x| QuotableBorrowedStr(x)));
        let take = QuotableOption(self.0.take.as_ref());
        let connection = QuotableOption(self.0.connection.as_ref().map(|x| QuotableBorrowedStr(x)));
        let direction = QuotableDirection(self.0.direction.clone());

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
