use crate::codegen::{
    quotable::{QuotableBorrowedStr, QuotableOption},
    AttributeArguments, TryFrom,
};
use crate::util::to_camel_case;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use serde::{ser::SerializeMap, Serialize, Serializer};
use std::borrow::Cow;
use syn::{spanned::Spanned, Lit};

pub const QUEUE_TYPE: &str = "queue";

#[derive(Debug, Clone)]
pub struct Queue {
    pub name: Cow<'static, str>,
    pub queue_name: Cow<'static, str>,
    pub connection: Option<Cow<'static, str>>,
}

// TODO: when https://github.com/serde-rs/serde/issues/760 is resolved, remove implementation in favor of custom Serialize derive
// The fix would allow us to set the constant `type` and `direction` entries rather than having to emit them manually.
impl Serialize for Queue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("type", QUEUE_TYPE)?;
        map.serialize_entry("direction", "out")?;
        map.serialize_entry("queueName", &self.queue_name)?;

        if let Some(connection) = self.connection.as_ref() {
            map.serialize_entry("connection", connection)?;
        }

        map.end()
    }
}

impl TryFrom<AttributeArguments> for Queue {
    type Error = (Span, String);

    fn try_from(args: AttributeArguments) -> Result<Self, Self::Error> {
        let mut name = None;
        let mut queue_name = None;
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
                "queue_name" => match value {
                    Lit::Str(s) => {
                        queue_name = Some(Cow::Owned(s.value()));
                    }
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal string value for the 'queue_name' argument"
                                .to_string(),
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

        if queue_name.is_none() {
            return Err((
                args.span,
                "the 'queue_name' argument is required for queue message bindings.".to_string(),
            ));
        }

        Ok(Queue {
            name: name.unwrap(),
            queue_name: queue_name.unwrap(),
            connection,
        })
    }
}

impl ToTokens for Queue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = QuotableBorrowedStr(&self.name);
        let queue_name = QuotableBorrowedStr(&self.queue_name);
        let connection = QuotableOption(self.connection.as_ref().map(|x| QuotableBorrowedStr(x)));

        quote!(::azure_functions::codegen::bindings::Queue {
            name: #name,
            queue_name: #queue_name,
            connection: #connection,
        })
        .to_tokens(tokens)
    }
}
