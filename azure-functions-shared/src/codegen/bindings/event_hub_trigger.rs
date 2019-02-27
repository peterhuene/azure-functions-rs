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

pub const EVENT_HUB_TRIGGER_TYPE: &str = "eventHubTrigger";

#[derive(Debug, Clone)]
pub struct EventHubTrigger {
    pub name: Cow<'static, str>,
    pub connection: Cow<'static, str>,
    pub event_hub_name: Option<Cow<'static, str>>,
    pub consumer_group: Option<Cow<'static, str>>,
}

// TODO: when https://github.com/serde-rs/serde/issues/760 is resolved, remove implementation in favor of custom Serialize derive
// The fix would allow us to set the constant `type` and `direction` entries rather than having to emit them manually.
impl Serialize for EventHubTrigger {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("type", EVENT_HUB_TRIGGER_TYPE)?;
        map.serialize_entry("direction", "in")?;
        map.serialize_entry("connection", &self.connection)?;

        if let Some(event_hub_name) = self.event_hub_name.as_ref() {
            map.serialize_entry("eventHubName", event_hub_name)?;
        }

        if let Some(consumer_group) = self.consumer_group.as_ref() {
            map.serialize_entry("consumerGroup", consumer_group)?;
        }

        map.end()
    }
}

impl TryFrom<AttributeArguments> for EventHubTrigger {
    type Error = (Span, String);

    fn try_from(args: AttributeArguments) -> Result<Self, Self::Error> {
        let mut name = None;
        let mut connection = None;
        let mut event_hub_name = None;
        let mut consumer_group = None;

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
                "event_hub_name" => match value {
                    Lit::Str(s) => {
                        event_hub_name = Some(Cow::Owned(s.value()));
                    }
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal string value for the 'event_hub_name' argument"
                                .to_string(),
                        ));
                    }
                },
                "consumer_group" => match value {
                    Lit::Str(s) => {
                        consumer_group = Some(Cow::Owned(s.value()));
                    }
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal string value for the 'consumer_group' argument"
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

        if connection.is_none() {
            return Err((
                args.span,
                "the 'connection' argument is required for Event Hub trigger bindings.".to_string(),
            ));
        }

        Ok(EventHubTrigger {
            name: name.unwrap(),
            connection: connection.unwrap(),
            event_hub_name,
            consumer_group,
        })
    }
}

impl ToTokens for EventHubTrigger {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = QuotableBorrowedStr(&self.name);
        let connection = QuotableBorrowedStr(&self.connection);
        let event_hub_name =
            QuotableOption(self.event_hub_name.as_ref().map(|x| QuotableBorrowedStr(x)));
        let consumer_group =
            QuotableOption(self.consumer_group.as_ref().map(|x| QuotableBorrowedStr(x)));

        quote!(::azure_functions::codegen::bindings::EventHubTrigger {
            name: #name,
            connection: #connection,
            event_hub_name: #event_hub_name,
            consumer_group: #consumer_group,
        })
        .to_tokens(tokens)
    }
}
