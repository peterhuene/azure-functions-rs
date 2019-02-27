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

pub const BLOB_TRIGGER_TYPE: &str = "blobTrigger";

#[derive(Debug, Clone)]
pub struct BlobTrigger {
    pub name: Cow<'static, str>,
    pub path: Cow<'static, str>,
    pub connection: Option<Cow<'static, str>>,
    pub direction: Direction,
}

// TODO: when https://github.com/serde-rs/serde/issues/760 is resolved, remove implementation in favor of custom Serialize derive
// The fix would allow us to set the constant `type` entry rather than having to emit it manually.
impl Serialize for BlobTrigger {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("type", BLOB_TRIGGER_TYPE)?;
        map.serialize_entry("direction", &self.direction)?;
        map.serialize_entry("path", &self.path)?;

        if let Some(connection) = self.connection.as_ref() {
            map.serialize_entry("connection", connection)?;
        }

        map.end()
    }
}

impl TryFrom<AttributeArguments> for BlobTrigger {
    type Error = (Span, String);

    fn try_from(args: AttributeArguments) -> Result<Self, Self::Error> {
        let mut name = None;
        let mut path = None;
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
                "path" => match value {
                    Lit::Str(s) => {
                        path = Some(Cow::Owned(s.value()));
                    }
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal string value for the 'path' argument".to_string(),
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

        if path.is_none() {
            return Err((
                args.span,
                "the 'path' argument is required for blob trigger bindings.".to_string(),
            ));
        }

        Ok(BlobTrigger {
            name: name.unwrap(),
            path: path.unwrap(),
            connection,
            direction: Direction::In,
        })
    }
}

impl ToTokens for BlobTrigger {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = QuotableBorrowedStr(&self.name);
        let path = QuotableBorrowedStr(&self.path);
        let connection = QuotableOption(self.connection.as_ref().map(|x| QuotableBorrowedStr(x)));
        let direction = QuotableDirection(self.direction.clone());

        quote!(::azure_functions::codegen::bindings::BlobTrigger {
            name: #name,
            path: #path,
            connection: #connection,
            direction: #direction,
        })
        .to_tokens(tokens)
    }
}
