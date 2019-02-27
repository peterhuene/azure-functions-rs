use crate::codegen::{quotable::QuotableBorrowedStr, AttributeArguments, TryFrom};
use crate::util::to_camel_case;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use serde::{ser::SerializeMap, Serialize, Serializer};
use std::borrow::Cow;
use syn::{spanned::Spanned, Lit};

pub const EVENT_GRID_TRIGGER_TYPE: &str = "eventGridTrigger";

#[derive(Debug, Clone)]
pub struct EventGridTrigger {
    pub name: Cow<'static, str>,
}

// TODO: when https://github.com/serde-rs/serde/issues/760 is resolved, remove implementation in favor of custom Serialize derive
// The fix would allow us to set the constant `type` and `direction` entries rather than having to emit them manually.
impl Serialize for EventGridTrigger {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("type", EVENT_GRID_TRIGGER_TYPE)?;
        map.serialize_entry("direction", "in")?;

        map.end()
    }
}

impl TryFrom<AttributeArguments> for EventGridTrigger {
    type Error = (Span, String);

    fn try_from(args: AttributeArguments) -> Result<Self, Self::Error> {
        let mut name = None;

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
                _ => {
                    return Err((
                        key.span(),
                        format!("unsupported binding attribute argument '{}'", key_str),
                    ));
                }
            };
        }

        Ok(EventGridTrigger {
            name: name.unwrap(),
        })
    }
}

impl ToTokens for EventGridTrigger {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = QuotableBorrowedStr(&self.name);

        quote!(::azure_functions::codegen::bindings::EventGridTrigger {
            name: #name,
        })
        .to_tokens(tokens)
    }
}
